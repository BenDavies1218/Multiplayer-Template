//! Character movement system (shared between client and server)

use avian3d::prelude::forces::ForcesItem;
use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::core_config::{CharacterConfig, GameCoreConfig, MovementConfig};
use crate::networking::protocol::{CharacterAction, CrouchState};

/// Apply camera-relative character movement.
/// Used by both client (predicted) and server (authoritative).
pub fn apply_character_movement(
    entity: Entity,
    mass: &ComputedMass,
    time: &Time,
    spatial_query: &SpatialQuery,
    action_state: &ActionState<CharacterAction>,
    mut forces: ForcesItem,
    camera_yaw: f32,
    crouch_state: &mut CrouchState,
    movement: &MovementConfig,
    character: &CharacterConfig,
) {
    let input = action_state
        .axis_pair(&CharacterAction::Move)
        .clamp_length_max(1.0);
    let wants_jump = action_state.just_pressed(&CharacterAction::Jump);
    let wants_sprint = action_state.pressed(&CharacterAction::Sprint);
    let wants_crouch = action_state.pressed(&CharacterAction::Crouch);

    // === GROUND CHECK ===
    let pos = forces.position().0;
    let current_capsule_height = if crouch_state.0 {
        movement.crouch_capsule_height
    } else {
        character.capsule_height
    };
    let capsule_half_extent = current_capsule_height / 2.0 + character.capsule_radius;
    let ground_tolerance = movement.ground_tolerance;
    let on_ground = spatial_query
        .cast_ray(
            pos,
            Dir3::NEG_Y,
            capsule_half_extent + ground_tolerance,
            true,
            &SpatialQueryFilter::from_excluded_entities([entity]),
        )
        .is_some();

    // === CROUCH / SPRINT STATE ===
    // Sprint requires ground and overrides crouch
    let is_sprinting = wants_sprint && on_ground;
    let is_crouching = wants_crouch && !is_sprinting;

    // Update crouch state (collider is updated separately to avoid SpatialQuery conflict)
    crouch_state.0 = is_crouching;

    // === JUMPING (blocked while crouching) ===
    if !is_crouching && wants_jump && on_ground {
        forces.apply_linear_impulse(Vec3::new(0.0, movement.jump_impulse, 0.0));
    }

    // === MOVEMENT (Camera-Relative) ===
    // Use max_deceleration when no input is held so the character stops snappily.
    let is_braking = input.length_squared() < 0.001;
    let accel = if is_braking {
        movement.max_deceleration
    } else {
        movement.max_acceleration
    };
    let max_velocity_delta_per_tick = accel * time.delta_secs();

    // Rotate movement direction by camera yaw
    let yaw_rotation = Quat::from_rotation_y(camera_yaw);
    let forward = yaw_rotation * Vec3::NEG_Z;
    let right = yaw_rotation * Vec3::X;

    // W/S moves in camera forward/back, A/D moves in camera left/right
    let move_dir = forward * input.y + right * input.x;

    // Get current horizontal velocity
    let linear_velocity = forces.linear_velocity();
    let ground_velocity = Vec3::new(linear_velocity.x, 0.0, linear_velocity.z);

    // Speed modifier based on state
    let speed = if is_sprinting {
        movement.max_speed * movement.sprint_multiplier
    } else if is_crouching {
        movement.max_speed * movement.crouch_multiplier
    } else {
        movement.max_speed
    };

    // Calculate desired velocity
    let desired_velocity = move_dir * speed;

    // Smoothly move toward desired velocity
    let new_velocity = ground_velocity.move_towards(desired_velocity, max_velocity_delta_per_tick);

    // Calculate required acceleration to reach new velocity
    let required_acceleration = (new_velocity - ground_velocity) / time.delta_secs();
    let force_vec = required_acceleration * mass.value();

    // Log each tick where the movement system is actively braking
    if is_braking && ground_velocity.length() > 0.05 {
        warn!(
            "[braking] {entity:?}  speed {:.4}→{:.4}  force=({:.3},{:.3})  mass={:.2}",
            ground_velocity.length(),
            new_velocity.length(),
            force_vec.x,
            force_vec.z,
            mass.value()
        );
    }

    // Apply force to achieve the acceleration
    forces.apply_force(force_vec);
}

/// Syncs the collider shape to match the current crouch state.
/// Run as a separate system to avoid conflicts with SpatialQuery.
pub fn update_crouch_collider(
    mut query: Query<(&CrouchState, &mut Collider), Changed<CrouchState>>,
    config: Res<GameCoreConfig>,
) {
    for (crouch_state, mut collider) in &mut query {
        if crouch_state.0 {
            *collider = Collider::capsule(
                config.character.capsule_radius,
                config.movement.crouch_capsule_height,
            );
        } else {
            *collider = Collider::capsule(
                config.character.capsule_radius,
                config.character.capsule_height,
            );
        }
    }
}
