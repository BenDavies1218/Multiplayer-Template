use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use lightyear::prelude::Controlled;
use lightyear::prelude::*;
use game_core::networking::protocol::*;
use game_core::networking::shared::*;
use game_core::movement::{apply_character_movement, update_crouch_collider};
use game_core::GameCoreConfig;
use game_camera::GameCamera;

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        // Sync camera BEFORE movement in FixedUpdate so it's sent as input
        app.add_systems(FixedUpdate, (sync_camera_to_character, handle_character_actions, update_crouch_collider).chain());
        app.add_systems(Update, handle_new_character);
    }
}

/// Process character actions and apply camera-relative movement
fn handle_character_actions(
    time: Res<Time>,
    spatial_query: SpatialQuery,
    mut query: Query<
        (Entity, &ComputedMass, &ActionState<CharacterAction>, Forces, &mut CrouchState),
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
) {
    for (entity, _color, is_controlled) in &mut character_query {
        if is_controlled {
            info!("Adding InputMap to controlled and predicted entity {entity:?}");
            commands.entity(entity).insert(
                InputMap::new([(CharacterAction::Jump, KeyCode::Space)])
                    .with(CharacterAction::Jump, GamepadButton::South)
                    .with(CharacterAction::Sprint, KeyCode::ShiftLeft)
                    .with(CharacterAction::Sprint, GamepadButton::LeftThumb)
                    .with(CharacterAction::Crouch, KeyCode::KeyC)
                    .with(CharacterAction::Crouch, GamepadButton::East)
                    .with(CharacterAction::Shoot, KeyCode::KeyQ)
                    .with_dual_axis(CharacterAction::Move, GamepadStick::LEFT)
                    .with_dual_axis(CharacterAction::Move, VirtualDPad::wasd()),
            );
        } else {
            info!("Remote character predicted for us: {entity:?}");
        }
        info!(?entity, "Adding physics to character");
        commands
            .entity(entity)
            .insert((
                CharacterPhysicsBundle::default(),
                CameraOrientation { yaw: 0.0, pitch: 0.0 },
                CrouchState::default(),
            ));
    }
}

fn sync_camera_to_character(
    camera_query: Query<&GameCamera>,
    mut character_query: Query<&mut ActionState<CharacterAction>, (With<CharacterMarker>, With<Predicted>, With<Controlled>)>,
) {
    let Ok(game_camera) = camera_query.single() else {
        return;
    };

    for mut action_state in &mut character_query {
        action_state.set_axis_pair(&CharacterAction::Look, Vec2::new(game_camera.yaw, game_camera.pitch));
    }
}