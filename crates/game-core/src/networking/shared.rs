use bevy::prelude::*;

use super::protocol::ProtocolPlugin;
use avian3d::prelude::*;
use bevy::math::EulerRot;
use core::time::Duration;
use leafwing_input_manager::prelude::ActionState;
use lightyear::avian3d::plugin::AvianReplicationMode;
use lightyear::prelude::*;

use crate::core_config::GameCoreConfig;
use crate::networking::protocol::{
    CameraOrientation, CharacterAction, CharacterMarker, ProjectileMarker,
};

#[derive(Bundle)]
pub struct CharacterPhysicsBundle {
    collider: Collider,
    rigid_body: RigidBody,
    lock_axes: LockedAxes,
    friction: Friction,
}

impl CharacterPhysicsBundle {
    /// Create a physics bundle using values from `CharacterConfig`.
    pub fn new(character: &crate::core_config::CharacterConfig) -> Self {
        Self {
            collider: Collider::capsule(character.capsule_radius, character.capsule_height),
            rigid_body: RigidBody::Dynamic,
            lock_axes: LockedAxes::default()
                .lock_rotation_x()
                .lock_rotation_y()
                .lock_rotation_z(),
            friction: Friction::new(0.0).with_combine_rule(CoefficientCombine::Min),
        }
    }
}

#[derive(Component)]
pub struct DespawnAfter {
    pub spawned_at: f32,
    pub lifetime: Duration,
}

pub fn despawn_system(
    mut commands: Commands,
    query: Query<(Entity, &DespawnAfter)>,
    time: Res<Time<Fixed>>,
) {
    for (entity, despawn) in &query {
        if time.elapsed_secs() - despawn.spawned_at >= despawn.lifetime.as_secs_f32() {
            commands.entity(entity).despawn();
        }
    }
}

fn shoot_bullet(
    mut commands: Commands,
    query: Query<
        (
            &ActionState<CharacterAction>,
            &Position,
            &CameraOrientation,
            Option<&ControlledBy>,
        ),
        (
            // With<Predicted> matches client-side predicted entities.
            // With<Replicate> matches server-side authoritative entities (Replicate is only present on server).
            Or<(With<Predicted>, With<Replicate>)>,
            With<CharacterMarker>,
        ),
    >,
    config: Res<GameCoreConfig>,
    time: Res<Time<Fixed>>,
) {
    for (action_state, position, orientation, controlled_by) in &query {
        if !action_state.just_pressed(&CharacterAction::Shoot) {
            continue;
        }

        let direction =
            Quat::from_euler(EulerRot::YXZ, orientation.yaw, orientation.pitch, 0.0)
                * Vec3::NEG_Z;
        let velocity = direction * config.projectile.velocity;

        let bullet_bundle = (
            Name::new("Projectile"),
            ProjectileMarker,
            RigidBody::Dynamic,
            *position,
            Rotation::default(),
            LinearVelocity(velocity),
        );

        if let Some(controlled_by) = controlled_by {
            // Server side — replicate to clients with replicate-once physics overrides
            let mut position_override = ComponentReplicationOverrides::<Position>::default();
            position_override.global_override(ComponentReplicationOverride {
                replicate_once: true,
                ..default()
            });
            let mut rotation_override = ComponentReplicationOverrides::<Rotation>::default();
            rotation_override.global_override(ComponentReplicationOverride {
                replicate_once: true,
                ..default()
            });
            let mut lv_override = ComponentReplicationOverrides::<LinearVelocity>::default();
            lv_override.global_override(ComponentReplicationOverride {
                replicate_once: true,
                ..default()
            });
            let mut av_override = ComponentReplicationOverrides::<AngularVelocity>::default();
            av_override.global_override(ComponentReplicationOverride {
                replicate_once: true,
                ..default()
            });

            commands.spawn((
                bullet_bundle,
                PreSpawned::default(),
                Replicate::to_clients(NetworkTarget::All),
                PredictionTarget::to_clients(NetworkTarget::All),
                ControlledBy {
                    owner: controlled_by.owner,
                    lifetime: Default::default(),
                },
                DespawnAfter {
                    spawned_at: time.elapsed_secs(),
                    lifetime: Duration::from_millis(config.projectile.lifetime_ms),
                },
                position_override,
                rotation_override,
                lv_override,
                av_override,
            ));
        } else {
            // Client side — pre-spawned local bullet; DespawnAfter added locally
            // since DespawnAfter is not replicated. Lightyear will match this entity
            // with the server's replicated bullet and won't remove local-only components.
            commands.spawn((
                bullet_bundle,
                PreSpawned::default(),
                DespawnAfter {
                    spawned_at: time.elapsed_secs(),
                    lifetime: Duration::from_millis(config.projectile.lifetime_ms),
                },
            ));
        }
    }
}

#[derive(Clone)]
pub struct SharedPlugin {
    pub config: GameCoreConfig,
}

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        // Initialize rollback thresholds from config
        crate::networking::protocol::init_rollback_config(self.config.rollback_thresholds.clone());

        // Insert GameCoreConfig as a resource for other systems to use
        app.insert_resource(self.config.clone());

        // Networking protocols
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

        app.add_systems(FixedUpdate, (shoot_bullet, despawn_system).chain());

        // WorldPlugin is added separately by each app with the appropriate config
    }
}
