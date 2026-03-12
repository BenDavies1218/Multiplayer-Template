//! Character movement system (shared between client and server)

use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::protocol::{CharacterAction, CrouchState};
use game_core::core_config::{CharacterConfig, GameCoreConfig, MovementConfig};

/// Apply camera-relative character movement via direct velocity setting.
/// Used by both client (predicted) and server (authoritative).
///
/// Sets LinearVelocity directly instead of applying forces — this makes
/// the simulation deterministic between client and server since both sides
/// compute `Quat::from_rotation_y(yaw) * input * speed` (pure math).
pub fn apply_character_movement(
    entity: Entity,
    linear_velocity: &mut LinearVelocity,
    spatial_query: &SpatialQuery,
    action_state: &ActionState<CharacterAction>,
    position: &Position,
    camera_yaw: f32,
    crouch_state: &mut CrouchState,
    movement: &MovementConfig,
    character: &CharacterConfig,
    fixed_timestep_hz: f32,
) {
    let input = action_state
        .axis_pair(&CharacterAction::Move)
        .clamp_length_max(1.0);
    let wants_jump = action_state.just_pressed(&CharacterAction::Jump);
    let wants_sprint = action_state.pressed(&CharacterAction::Sprint);
    let wants_crouch = action_state.pressed(&CharacterAction::Crouch);

    // === GROUND CHECK ===
    let current_capsule_height = if crouch_state.0 {
        movement.crouch_capsule_height
    } else {
        character.capsule_height
    };
    let capsule_half_extent = current_capsule_height / 2.0 + character.capsule_radius;
    let on_ground = spatial_query
        .cast_ray(
            position.0,
            Dir3::NEG_Y,
            capsule_half_extent + movement.ground_tolerance,
            true,
            &SpatialQueryFilter::from_excluded_entities([entity]),
        )
        .is_some();

    // === CROUCH / SPRINT STATE ===
    // Sprint requires forward input (W key) — no sprinting while only strafing.
    let is_sprinting = wants_sprint && on_ground && input.y > 0.0;
    let is_crouching = wants_crouch && !is_sprinting;
    crouch_state.0 = is_crouching;

    // === JUMP (blocked while crouching) ===
    if !is_crouching && wants_jump && on_ground {
        linear_velocity.y = movement.jump_impulse;
    }

    // === MOVEMENT (Camera-Relative) ===
    let yaw_rotation = Quat::from_rotation_y(camera_yaw);
    let forward = yaw_rotation * Vec3::NEG_Z;
    let right = yaw_rotation * Vec3::X;
    let move_dir = forward * input.y + right * input.x;

    // Speed modifier based on state
    let speed = if is_sprinting {
        movement.max_speed * movement.sprint_multiplier
    } else if is_crouching {
        movement.max_speed * movement.crouch_multiplier
    } else {
        movement.max_speed
    };

    // Smooth acceleration/deceleration via move_towards.
    // Uses a hardcoded dt (1/fixed_timestep_hz) instead of time.delta_secs()
    // to guarantee identical results on client and server — pure math,
    // no dependency on frame timing variance.
    let dt = 1.0 / fixed_timestep_hz;
    let desired = move_dir * speed;
    let current_xz = Vec3::new(linear_velocity.x, 0.0, linear_velocity.z);
    let target_xz = Vec3::new(desired.x, 0.0, desired.z);
    let is_braking = desired.length_squared() < current_xz.length_squared();
    let accel = if is_braking {
        movement.max_deceleration
    } else {
        movement.max_acceleration
    };
    let new_xz = current_xz.move_towards(target_xz, accel * dt);
    linear_velocity.x = new_xz.x;
    linear_velocity.z = new_xz.z;
}

/// Syncs the collider shape to match the current crouch state.
/// Run as a separate system to avoid conflicts with SpatialQuery.
pub fn update_crouch_collider(
    mut query: Query<(Entity, &CrouchState, &mut Collider), Changed<CrouchState>>,
    config: Res<GameCoreConfig>,
) {
    for (entity, crouch_state, mut collider) in &mut query {
        if crouch_state.0 {
            debug!(
                "[CROUCH-COLLIDER] {entity:?} crouching=true → capsule h={:.3}",
                config.movement.crouch_capsule_height
            );
            *collider = Collider::capsule(
                config.character.capsule_radius,
                config.movement.crouch_capsule_height,
            );
        } else {
            debug!(
                "[CROUCH-COLLIDER] {entity:?} crouching=false → capsule h={:.3}",
                config.character.capsule_height
            );
            *collider = Collider::capsule(
                config.character.capsule_radius,
                config.character.capsule_height,
            );
        }
    }
}
