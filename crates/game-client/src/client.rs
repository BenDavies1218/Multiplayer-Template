use std::collections::HashMap;

use crate::client_config::{GameClientConfig, parse_gamepad_button};
use avian3d::prelude::*;
use bevy::prelude::*;
use game_camera::GameCamera;
use game_core::GameCoreConfig;
use game_core::core_config::parse_key_code;
use game_core::movement::{apply_character_movement, update_crouch_collider};
use game_core::networking::protocol::{self, *};
use game_core::networking::shared::*;
use leafwing_input_manager::prelude::*;
use lightyear::prelude::Controlled;
use lightyear::prelude::*;
use lightyear::prelude::is_in_rollback;

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        // track_velocity_changes runs FIRST to capture velocity at start of tick,
        // before any forces are applied — lets us see rollback-induced velocity jumps.
        app.add_systems(
            FixedUpdate,
            (
                update_prediction_speed,
                log_rollback_tick.run_if(is_in_rollback),
                track_velocity_changes,
                // Only sync camera yaw into ActionState during normal ticks.
                // During rollback re-simulation Lightyear restores the historical
                // ActionState (including the Look axis the client sent at that tick).
                // Running this system during rollback would overwrite that historical
                // yaw with the *current* camera angle, causing a direction mismatch
                // vs the server — triggering a position rollback every stop.
                sync_camera_to_character.run_if(not(is_in_rollback)),
                handle_character_actions,
                update_crouch_collider,
            )
                .chain(),
        );
        app.add_systems(Update, handle_new_character);
    }
}

/// Updates the global speed used by `position_should_rollback` to scale the threshold.
/// Must run at the start of each FixedUpdate tick so the threshold is current.
fn update_prediction_speed(
    query: Query<&LinearVelocity, (With<Predicted>, With<CharacterMarker>)>,
) {
    let max_speed = query
        .iter()
        .map(|v| Vec2::new(v.x, v.z).length())
        .fold(0.0f32, f32::max);
    protocol::set_prediction_speed(max_speed);
}

/// Logs every FixedUpdate tick that runs during Lightyear rollback re-simulation.
/// Confirms that is_in_rollback detection is working.
fn log_rollback_tick(
    timeline: Res<LocalTimeline>,
    query: Query<(Entity, &LinearVelocity), With<Predicted>>,
) {
    let tick = timeline.tick();
    for (entity, vel) in &query {
        let speed = Vec2::new(vel.x, vel.z).length();
        warn!("[ROLLBACK-TICK] tick={tick:?} {entity:?} speed={speed:.3}");
    }
}

/// Runs before movement each tick.
/// Compares current velocity to the velocity stored from the end of last tick.
/// A velocity INCREASE (any amount > 0.1 m/s) from the previous tick is a rollback signature —
/// physics forces only decrease or maintain speed, jumps come from state restoration.
fn track_velocity_changes(
    mut last_speeds: Local<HashMap<Entity, f32>>,
    query: Query<(Entity, &LinearVelocity, &ActionState<CharacterAction>), With<Predicted>>,
) {
    for (entity, vel, action_state) in &query {
        let horiz_speed = Vec2::new(vel.x, vel.z).length();
        let input = action_state
            .axis_pair(&CharacterAction::Move)
            .clamp_length_max(1.0);
        let look = action_state.axis_pair(&CharacterAction::Look);
        let is_braking = input.length_squared() < 0.001;

        if let Some(&prev_speed) = last_speeds.get(&entity) {
            if prev_speed > 0.05 || horiz_speed > 0.05 {
                if horiz_speed > prev_speed + 0.1 {
                    warn!(
                        "[VEL-JUMP] {entity:?} speed {prev_speed:.3}→{horiz_speed:.3} \
                         vel=({:.3},{:.3})  input=({:.3},{:.3})  look_yaw={:.3}  braking={is_braking} \
                         — VELOCITY JUMPED (likely rollback)",
                        vel.x, vel.z, input.x, input.y, look.x
                    );
                } else {
                    warn!(
                        "[vel-tick] {entity:?} speed {prev_speed:.3}→{horiz_speed:.3} \
                         input=({:.3},{:.3})  look_yaw={:.3}  braking={is_braking}",
                        input.x, input.y, look.x
                    );
                }
            }
        }

        last_speeds.insert(entity, horiz_speed);
    }
}

/// Process character actions and apply camera-relative movement
fn handle_character_actions(
    time: Res<Time>,
    spatial_query: SpatialQuery,
    mut query: Query<
        (
            Entity,
            &ComputedMass,
            &ActionState<CharacterAction>,
            Forces,
            &mut CrouchState,
        ),
        With<Predicted>,
    >,
    config: Res<GameCoreConfig>,
) {
    for (entity, computed_mass, action_state, forces, mut crouch_state) in &mut query {
        // Get camera yaw from the Look action
        let camera_yaw = action_state.axis_pair(&CharacterAction::Look).x;

        apply_character_movement(
            entity,
            computed_mass,
            &time,
            &spatial_query,
            action_state,
            forces,
            camera_yaw,
            &mut crouch_state,
            &config.movement,
            &config.character,
        );
    }
}

fn handle_new_character(
    mut commands: Commands,
    mut character_query: Query<
        (Entity, &ColorComponent, Has<Controlled>),
        (Added<Predicted>, With<CharacterMarker>),
    >,
    client_config: Res<GameClientConfig>,
    core_config: Res<GameCoreConfig>,
) {
    for (entity, _color, is_controlled) in &mut character_query {
        if is_controlled {
            let jump_key = parse_key_code(&client_config.input.jump_key).unwrap_or(KeyCode::Space);
            let sprint_key =
                parse_key_code(&client_config.input.sprint_key).unwrap_or(KeyCode::ShiftLeft);
            let crouch_key =
                parse_key_code(&client_config.input.crouch_key).unwrap_or(KeyCode::KeyC);
            let shoot_key = parse_key_code(&client_config.input.shoot_key).unwrap_or(KeyCode::KeyQ);
            let jump_gamepad = parse_gamepad_button(&client_config.input.jump_gamepad)
                .unwrap_or(GamepadButton::South);
            let sprint_gamepad = parse_gamepad_button(&client_config.input.sprint_gamepad)
                .unwrap_or(GamepadButton::LeftThumb);
            let crouch_gamepad = parse_gamepad_button(&client_config.input.crouch_gamepad)
                .unwrap_or(GamepadButton::East);

            info!("Adding InputMap to controlled and predicted entity {entity:?}");
            commands.entity(entity).insert(
                InputMap::new([(CharacterAction::Jump, jump_key)])
                    .with(CharacterAction::Jump, jump_gamepad)
                    .with(CharacterAction::Sprint, sprint_key)
                    .with(CharacterAction::Sprint, sprint_gamepad)
                    .with(CharacterAction::Crouch, crouch_key)
                    .with(CharacterAction::Crouch, crouch_gamepad)
                    .with(CharacterAction::Shoot, shoot_key)
                    .with(CharacterAction::Shoot, MouseButton::Left)
                    .with_dual_axis(CharacterAction::Move, GamepadStick::LEFT)
                    .with_dual_axis(CharacterAction::Move, VirtualDPad::wasd()),
            );
        } else {
            info!("Remote character predicted for us: {entity:?}");
        }
        info!(?entity, "Adding physics to character");
        commands.entity(entity).insert((
            CharacterPhysicsBundle::new(&core_config.character),
            CameraOrientation {
                yaw: 0.0,
                pitch: 0.0,
            },
            CrouchState::default(),
        ));
    }
}

fn sync_camera_to_character(
    camera_query: Query<&GameCamera>,
    mut character_query: Query<
        &mut ActionState<CharacterAction>,
        (With<CharacterMarker>, With<Predicted>, With<Controlled>),
    >,
) {
    let Ok(game_camera) = camera_query.single() else {
        return;
    };

    for mut action_state in &mut character_query {
        action_state.set_axis_pair(
            &CharacterAction::Look,
            Vec2::new(game_camera.yaw, game_camera.pitch),
        );
    }
}
