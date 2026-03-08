//! Character movement system (shared between client and server)

use avian3d::prelude::*;
use avian3d::prelude::forces::ForcesItem;
use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::protocol::{CharacterAction, CrouchState};
use crate::shared::{CHARACTER_CAPSULE_HEIGHT, CHARACTER_CAPSULE_RADIUS};

const MAX_SPEED: f32 = 5.0;
const MAX_ACCELERATION: f32 = 20.0;
const JUMP_IMPULSE: f32 = 5.0;
const SPRINT_MULTIPLIER: f32 = 1.8;
const CROUCH_MULTIPLIER: f32 = 0.4;
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
) {
    // How much velocity can change in a single tick
    let max_velocity_delta_per_tick = MAX_ACCELERATION * time.delta_secs();

    // === CROUCH / SPRINT STATE ===
    let is_sprinting = action_state.pressed(&CharacterAction::Sprint);
    let wants_crouch = action_state.pressed(&CharacterAction::Crouch);
    // Sprint overrides crouch
    let is_crouching = wants_crouch && !is_sprinting;

    // Update crouch state (collider is updated separately to avoid SpatialQuery conflict)
    crouch_state.0 = is_crouching;

    let current_capsule_height = if is_crouching { CROUCH_CAPSULE_HEIGHT } else { CHARACTER_CAPSULE_HEIGHT };

    // === JUMPING (blocked while crouching) ===
    if !is_crouching && action_state.just_pressed(&CharacterAction::Jump) {
        let pos = forces.position().0;
        let ray_cast_origin = pos
            + Vec3::new(
                0.0,
                -current_capsule_height / 2.0 - CHARACTER_CAPSULE_RADIUS,
                0.0,
            );

        // Only jump if on the ground
        let hit = spatial_query
            .cast_ray(
                ray_cast_origin,
                Dir3::NEG_Y,
                0.15,
                true,
                &SpatialQueryFilter::from_excluded_entities([entity]),
            );

        info!(
            "JUMP DEBUG: pos={pos:?}, ray_origin={ray_cast_origin:?}, capsule_h={current_capsule_height}, radius={CHARACTER_CAPSULE_RADIUS}, hit={hit:?}"
        );

        if hit.is_some() {
            forces.apply_linear_impulse(Vec3::new(0.0, JUMP_IMPULSE, 0.0));
        }
    }

    // === MOVEMENT (Camera-Relative) ===
    let input = action_state.axis_pair(&CharacterAction::Move).clamp_length_max(1.0);

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
        MAX_SPEED * SPRINT_MULTIPLIER
    } else if is_crouching {
        MAX_SPEED * CROUCH_MULTIPLIER
    } else {
        MAX_SPEED
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
) {
    for (crouch_state, mut collider) in &mut query {
        if crouch_state.0 {
            *collider = Collider::capsule(CHARACTER_CAPSULE_RADIUS, CROUCH_CAPSULE_HEIGHT);
        } else {
            *collider = Collider::capsule(CHARACTER_CAPSULE_RADIUS, CHARACTER_CAPSULE_HEIGHT);
        }
    }
}
