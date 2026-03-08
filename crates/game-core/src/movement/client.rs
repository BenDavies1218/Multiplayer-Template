use avian3d::prelude::*;
use avian3d::prelude::forces::ForcesItem;
use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::protocol::CharacterAction;
use crate::shared::{CHARACTER_CAPSULE_HEIGHT, CHARACTER_CAPSULE_RADIUS};

const MAX_SPEED: f32 = 5.0;
const MAX_ACCELERATION: f32 = 20.0;
const JUMP_IMPULSE: f32 = 5.0;

/// Apply character movement on the client.
/// Camera-relative WASD movement for better controls.
pub fn apply_client_movement(
    entity: Entity,
    mass: &ComputedMass,
    time: &Time,
    spatial_query: &SpatialQuery,
    action_state: &ActionState<CharacterAction>,
    mut forces: ForcesItem,
    camera_yaw: f32,
) {
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
            info!("CLIENT JUMP: entity={:?}", entity);
        }
    }

    // === MOVEMENT (Camera-Relative) ===
    let input = action_state.axis_pair(&CharacterAction::Move).clamp_length_max(1.0);

    // Rotate movement direction by camera yaw
    let yaw_rotation = Quat::from_rotation_y(camera_yaw);
    let forward = yaw_rotation * Vec3::NEG_Z;  // Camera forward
    let right = yaw_rotation * Vec3::X;        // Camera right

    // W/S moves in camera forward/back, A/D moves in camera left/right
    let move_dir = forward * input.y + right * input.x;

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
