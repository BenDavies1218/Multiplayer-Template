use bevy::prelude::*;
use bevy::math::VectorSpace;

use crate::protocol::{ProtocolPlugin, CharacterAction};
use avian3d::prelude::*;
use avian3d::prelude::forces::ForcesItem;
use leafwing_input_manager::prelude::ActionState;
use lightyear::avian3d::plugin::AvianReplicationMode;

pub const CHARACTER_CAPSULE_RADIUS: f32 = 0.5;
pub const CHARACTER_CAPSULE_HEIGHT: f32 = 0.5;

// World asset constants
pub const WORLD_VISUAL_PATH: &str = "models/example_world_visual.glb";
pub const WORLD_COLLISION_PATH: &str = "models/example_world_collision.glb";

#[derive(Bundle)]
pub struct CharacterPhysicsBundle {
    collider: Collider,
    rigid_body: RigidBody,
    lock_axes: LockedAxes,
    friction: Friction,
}

impl Default for CharacterPhysicsBundle {
    fn default() -> Self {
        Self {
            collider: Collider::capsule(CHARACTER_CAPSULE_RADIUS, CHARACTER_CAPSULE_HEIGHT),
            rigid_body: RigidBody::Dynamic,
            lock_axes: LockedAxes::default()
                .lock_rotation_x()
                .lock_rotation_y()
                .lock_rotation_z(),
            friction: Friction::new(0.0).with_combine_rule(CoefficientCombine::Min),
        }
    }
}

#[derive(Clone)]
pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        // Neetworking protocols
        app.add_plugins(ProtocolPlugin);

        // types needed for replication and interpolation
        app.register_type::<Transform>();
        app.register_type::<GlobalTransform>();

        // Physics
        app.add_plugins(lightyear::avian3d::plugin::LightyearAvianPlugin {
            replication_mode: AvianReplicationMode::Position,
            ..default()
        });
        app.add_plugins(
            PhysicsPlugins::default()
                .build()
                .disable::<PhysicsTransformPlugin>()
                .disable::<PhysicsInterpolationPlugin>()
                .disable::<IslandPlugin>()
                .disable::<IslandSleepingPlugin>(),
        );

        // World loading (visual and collision)
        // Configure based on whether this is server-only or has client features
        #[cfg(all(feature = "server", not(feature = "client")))]
        {
            // Server-only: collision only, no visuals, no debug
            app.add_plugins(crate::world::WorldPlugin {
                config: crate::world::WorldPluginConfig::server(),
            });
            info!("WorldPlugin configured for server (collision only)");
        }

        #[cfg(feature = "client")]
        {
            // Client or hybrid: full features with debug
            app.add_plugins(crate::world::WorldPlugin {
                config: crate::world::WorldPluginConfig::client(),
            });
            info!("WorldPlugin configured for client (visual + collision + debug)");
        }
    }
}

/// Apply character movement (shared between client and server)
/// This uses world-space movement for simplicity and determinism
pub fn apply_character_movement(
    entity: Entity,
    mass: &ComputedMass,
    time: &Res<Time>,
    spatial_query: &SpatialQuery,
    action_state: &ActionState<CharacterAction>,
    mut forces: ForcesItem,
) {
    const MAX_SPEED: f32 = 5.0;
    const MAX_ACCELERATION: f32 = 20.0;
    const JUMP_IMPULSE: f32 = 5.0;

    // How much velocity can change in a single tick
    let max_velocity_delta_per_tick = MAX_ACCELERATION * time.delta_secs();

    // === JUMPING ===
    if action_state.just_pressed(&CharacterAction::Jump) {
        let ray_cast_origin = forces.position().0
            + Vec3::new(
                0.0,
                -CHARACTER_CAPSULE_HEIGHT / 2.0 - CHARACTER_CAPSULE_RADIUS,
                0.0,
            );

        // Only jump if on the ground
        if spatial_query
            .cast_ray(
                ray_cast_origin,
                Dir3::NEG_Y,
                0.01,
                true,
                &SpatialQueryFilter::from_excluded_entities([entity]),
            )
            .is_some()
        {
            forces.apply_linear_impulse(Vec3::new(0.0, JUMP_IMPULSE, 0.0));
        }
    }

    // === MOVEMENT (World-Space) ===
    // This is simpler than camera-relative movement and works identically on client/server
    let input = action_state
        .axis_pair(&CharacterAction::Move)
        .clamp_length_max(1.0);

    // World-space movement: input.x = strafe left/right, input.y = forward/back
    let move_dir = Vec3::new(-input.x, 0.0, input.y);

    // Get current horizontal velocity
    let linear_velocity = forces.linear_velocity();
    let ground_velocity = Vec3::new(linear_velocity.x, 0.0, linear_velocity.z);

    // Calculate desired velocity
    let desired_velocity = move_dir * MAX_SPEED;

    // Smoothly move toward desired velocity
    let new_velocity = ground_velocity.move_towards(desired_velocity, max_velocity_delta_per_tick);

    // Calculate required acceleration to reach new velocity
    let required_acceleration = (new_velocity - ground_velocity) / time.delta_secs();

    // Apply force to achieve the acceleration
    forces.apply_force(required_acceleration * mass.value());
}
