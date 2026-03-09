//! Character movement system (shared between client and server)

use avian3d::prelude::*;
use avian3d::prelude::forces::ForcesItem;
use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::core_config::{GameCoreConfig, MovementConfig, CharacterConfig};
use crate::networking::protocol::{CharacterAction, CrouchState};

// DEPRECATED: kept for backward compatibility (game-client renderer imports it).
// Will be removed in Task 11.
pub const CROUCH_CAPSULE_HEIGHT: f32 = 0.25;

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
    let input = action_state.axis_pair(&CharacterAction::Move).clamp_length_max(1.0);
    let wants_jump = action_state.just_pressed(&CharacterAction::Jump);
    let wants_sprint = action_state.pressed(&CharacterAction::Sprint);
    let wants_crouch = action_state.pressed(&CharacterAction::Crouch);

    // === GROUND CHECK ===
    let pos = forces.position().0;
    let current_capsule_height = if crouch_state.0 { movement.crouch_capsule_height } else { character.capsule_height };
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
    let max_velocity_delta_per_tick = movement.max_acceleration * time.delta_secs();

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

    // Apply force to achieve the acceleration
    forces.apply_force(required_acceleration * mass.value());
}

/// Syncs the collider shape to match the current crouch state.
/// Run as a separate system to avoid conflicts with SpatialQuery.
pub fn update_crouch_collider(
    mut query: Query<(&CrouchState, &mut Collider), Changed<CrouchState>>,
    config: Res<GameCoreConfig>,
) {
    for (crouch_state, mut collider) in &mut query {
        if crouch_state.0 {
            *collider = Collider::capsule(config.character.capsule_radius, config.movement.crouch_capsule_height);
        } else {
            *collider = Collider::capsule(config.character.capsule_radius, config.character.capsule_height);
        }
    }
}
