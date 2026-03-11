use avian3d::prelude::*;
use bevy::prelude::*;
use game_core::networking::settings::send_interval_from_config;
use leafwing_input_manager::prelude::*;
use lightyear::connection::client::Connected;
use lightyear::prelude::server::*;
use lightyear::prelude::*;

use game_core::GameCoreConfig;
use game_core::character::{CharacterHitboxData, CharacterModelId, attach_hitbox_to_character};
use game_core::movement::{apply_character_movement, update_crouch_collider};
use game_core::networking::protocol::*;
use game_core::networking::shared::CharacterPhysicsBundle;
use game_core::zones::SpawnPoints;

use crate::server_config::{GameServerConfig, parse_css_color};

#[derive(Clone)]
pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (handle_character_actions, update_crouch_collider).chain(),
        );
        app.add_observer(handle_new_client);
        app.add_observer(handle_connected);
    }
}

fn handle_character_actions(
    time: Res<Time>,
    spatial_query: SpatialQuery,
    mut query: Query<(
        Entity,
        &ComputedMass,
        &ActionState<CharacterAction>,
        &mut CameraOrientation,
        Forces,
        &mut CrouchState,
    )>,
    config: Res<GameCoreConfig>,
) {
    for (entity, mass, action_state, mut camera_orientation, forces, mut crouch_state) in &mut query
    {
        let look = action_state.axis_pair(&CharacterAction::Look);
        camera_orientation.yaw = look.x;
        camera_orientation.pitch = look.y;
        apply_character_movement(
            entity,
            mass,
            &time,
            &spatial_query,
            action_state,
            forces,
            camera_orientation.yaw,
            &mut crouch_state,
            &config.movement,
            &config.character,
        );
    }
}


/// Add the ReplicationSender component to new clients
pub(crate) fn handle_new_client(
    trigger: On<Add, LinkOf>,
    mut commands: Commands,
    config: Res<GameCoreConfig>,
) {
    let interval = send_interval_from_config(&config);
    commands
        .entity(trigger.entity)
        .insert(ReplicationSender::new(
            interval,
            SendUpdatesMode::SinceLastAck,
            false,
        ));
}

/// Spawn the player entity when a client connects
pub(crate) fn handle_connected(
    trigger: On<Add, Connected>,
    query: Query<&RemoteId, With<ClientOf>>,
    mut commands: Commands,
    character_query: Query<Entity, With<CharacterMarker>>,
    mut spawn_points: Option<ResMut<SpawnPoints>>,
    hitbox_data: Option<Res<CharacterHitboxData>>,
    server_config: Res<GameServerConfig>,
    core_config: Res<GameCoreConfig>,
) {
    let Ok(client_id) = query.get(trigger.entity) else {
        return;
    };
    let client_id = client_id.0;
    info!("Client connected with client-id {client_id:?}. Spawning character entity.");

    // Track the number of characters to pick colors and starting positions.
    let num_characters = character_query.iter().count();

    // Pick color from config.
    let available_colors: Vec<Color> = server_config
        .spawning
        .player_colors
        .iter()
        .map(|name| parse_css_color(name))
        .collect();
    let color = if available_colors.is_empty() {
        Color::WHITE
    } else {
        available_colors[num_characters % available_colors.len()]
    };

    // Use SpawnPoints if available, fallback to circular pattern from config
    let spawn_pos = if let Some(ref mut sp) = spawn_points {
        sp.next()
    } else {
        let angle: f32 = num_characters as f32 * server_config.spawning.fallback_angle_multiplier;
        let x = server_config.spawning.fallback_radius * angle.cos();
        let z = server_config.spawning.fallback_radius * angle.sin();
        Vec3::new(x, server_config.spawning.fallback_height, z)
    };

    // Spawn the character with ActionState. The client will add their own InputMap.
    let character = commands
        .spawn((
            Name::new("Character"),
            ActionState::<CharacterAction>::default(),
            Position(spawn_pos),
            Replicate::to_clients(NetworkTarget::All),
            PredictionTarget::to_clients(NetworkTarget::All),
            ControlledBy {
                owner: trigger.entity,
                lifetime: Default::default(),
            },
            CharacterPhysicsBundle::new(&core_config.character),
            ColorComponent(color),
            CharacterMarker,
            CameraOrientation {
                yaw: 0.0,
                pitch: 0.0,
            },
            CrouchState::default(),
        ))
        .id();

    // Add character model ID
    commands
        .entity(character)
        .insert(CharacterModelId::default());

    // Attach hitbox colliders as children if data is loaded
    if let Some(ref hitbox) = hitbox_data {
        attach_hitbox_to_character(&mut commands, character, hitbox);
        info!(
            "Attached {} hitbox regions to character {character:?}",
            hitbox.regions.len()
        );
    } else {
        warn!("CharacterHitboxData not yet loaded — character spawned without hitbox");
    }

    info!("Created entity {character:?} for client {client_id:?}");
}
