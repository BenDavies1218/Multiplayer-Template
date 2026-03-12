//! Player connection and character spawning.
//!
//! Two observers drive the lifecycle:
//! - `handle_new_client` — attaches a `ReplicationSender` when a link opens
//! - `handle_connected`  — spawns the character when the client fully connects

use avian3d::prelude::Position;
use bevy::prelude::*;
use game_core::GameCoreConfig;
use game_core::character::{CharacterHitboxData, CharacterModelId, attach_hitbox_to_character};
use game_core::networking::protocol::*;
use game_core::networking::settings::send_interval_from_config;
use game_core::networking::shared::CharacterPhysicsBundle;
use game_core::zones::SpawnPoints;
use leafwing_input_manager::prelude::*;
use lightyear::connection::client::Connected;
use lightyear::prelude::server::*;
use lightyear::prelude::*;

use crate::server_config::{GameServerConfig, parse_css_color};

/// Attach a `ReplicationSender` to new client links so state is replicated.
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
    debug!("[spawning] new client link {entity:?} — ReplicationSender attached", entity = trigger.entity);
}

/// Spawn a character entity when a client finishes connecting.
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

    let num_characters = character_query.iter().count();

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

    let spawn_pos = if let Some(ref mut sp) = spawn_points {
        sp.next()
    } else {
        let angle = num_characters as f32 * server_config.spawning.fallback_angle_multiplier;
        let x = server_config.spawning.fallback_radius * angle.cos();
        let z = server_config.spawning.fallback_radius * angle.sin();
        Vec3::new(x, server_config.spawning.fallback_height, z)
    };

    info!(
        "[spawning] client {client_id:?} connected — spawning character at ({:.2},{:.2},{:.2})",
        spawn_pos.x, spawn_pos.y, spawn_pos.z
    );

    let character = commands
        .spawn((
            Name::new("Character"),
            ActionState::<CharacterAction>::default(),
            Position(spawn_pos),
            Replicate::to_clients(NetworkTarget::All),
            // Owner gets client-side prediction (has the real inputs).
            // All OTHER clients get smooth interpolation — they don't have this
            // player's inputs, so prediction would always diverge and trigger
            // cascading rollbacks for the owner too.
            PredictionTarget::to_clients(NetworkTarget::Single(client_id)),
            InterpolationTarget::to_clients(NetworkTarget::AllExceptSingle(client_id)),
            ControlledBy {
                owner: trigger.entity,
                lifetime: Default::default(),
            },
            CharacterPhysicsBundle::new(&core_config.character),
            ColorComponent(color),
            CharacterMarker,
            CameraOrientation { yaw: 0.0, pitch: 0.0 },
            CrouchState::default(),
            // Prevent hitbox children from being replicated to clients.
            // Without this, Lightyear replicates child entities with Predicted +
            // Position but no movement system, creating "ghost" entities that
            // diverge from the server and trigger constant rollbacks.
            DisableReplicateHierarchy,
        ))
        .id();

    commands.entity(character).insert(CharacterModelId::default());

    if let Some(ref hitbox) = hitbox_data {
        attach_hitbox_to_character(&mut commands, character, hitbox);
        info!(
            "[spawning] attached {} hitbox regions to {character:?}",
            hitbox.regions.len()
        );
    } else {
        warn!("[spawning] CharacterHitboxData not loaded — character {character:?} spawned without hitbox");
    }

    info!("[spawning] entity {character:?} created for client {client_id:?}");
}
