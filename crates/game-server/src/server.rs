use avian3d::prelude::*;
use bevy::prelude::*;
use core::time::Duration;
use leafwing_input_manager::prelude::*;
use lightyear::connection::client::Connected;
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use game_core::networking::settings::send_interval_from_config;

use game_core::networking::protocol::*;
use game_core::networking::shared::CharacterPhysicsBundle;
use game_core::movement::{apply_character_movement, update_crouch_collider};
use game_core::zones::SpawnPoints;
use game_core::GameCoreConfig;

use crate::server_config::{GameServerConfig, parse_css_color};

#[derive(Clone)]
pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (handle_character_actions, update_crouch_collider, player_shoot, despawn_system).chain(),
        );
        app.add_observer(handle_new_client);
        app.add_observer(handle_connected);
    }
}

fn handle_character_actions(
    time: Res<Time>,
    spatial_query: SpatialQuery,
    mut query: Query<(Entity, &ComputedMass, &ActionState<CharacterAction>, &mut CameraOrientation, Forces, &mut CrouchState)>,
    config: Res<GameCoreConfig>,
) {
    for (entity, mass, action_state, mut camera_orientation, forces, mut crouch_state) in &mut query {
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

#[derive(Component)]
pub struct DespawnAfter {
    spawned_at: f32,
    lifetime: Duration,
}

fn despawn_system(
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

fn player_shoot(
    mut commands: Commands,
    _timeline: Res<LocalTimeline>,
    query: Query<(&ActionState<CharacterAction>, &Position, &ControlledBy), Without<Predicted>>,
    time: Res<Time<Fixed>>,
    server_config: Res<GameServerConfig>,
) {
    for (action_state, position, controlled_by) in &query {
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
        let mut linear_velocity_override =
            ComponentReplicationOverrides::<LinearVelocity>::default();
        linear_velocity_override.global_override(ComponentReplicationOverride {
            replicate_once: true,
            ..default()
        });
        let mut angular_velocity_override =
            ComponentReplicationOverrides::<AngularVelocity>::default();
        angular_velocity_override.global_override(ComponentReplicationOverride {
            replicate_once: true,
            ..default()
        });
        let mut computed_mass_override = ComponentReplicationOverrides::<ComputedMass>::default();
        computed_mass_override.global_override(ComponentReplicationOverride {
            replicate_once: true,
            ..default()
        });

        if action_state.just_pressed(&CharacterAction::Shoot) {
            commands.spawn((
                Name::new("Projectile"),
                ProjectileMarker,
                DespawnAfter {
                    spawned_at: time.elapsed_secs(),
                    lifetime: Duration::from_millis(server_config.projectile.lifetime_ms),
                },
                RigidBody::Dynamic,
                *position, // Use current position
                Rotation::default(),
                LinearVelocity(Vec3::Z * server_config.projectile.velocity),
                Replicate::to_clients(NetworkTarget::All),
                PredictionTarget::to_clients(NetworkTarget::All),
                ControlledBy {
                    owner: controlled_by.owner,
                    lifetime: Default::default(),
                },
                // we don't want clients to receive any replication updates after the initial spawn
                (
                    position_override,
                    rotation_override,
                    linear_velocity_override,
                    angular_velocity_override,
                    computed_mass_override,
                ),
            ));
        }
    }
}

/// Add the ReplicationSender component to new clients
pub(crate) fn handle_new_client(trigger: On<Add, LinkOf>, mut commands: Commands, config: Res<GameCoreConfig>) {
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
    let available_colors: Vec<Color> = server_config.spawning.player_colors
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
            CameraOrientation { yaw: 0.0, pitch: 0.0 },
            CrouchState::default(),
        ))
        .id();

    info!("Created entity {character:?} for client {client_id:?}");
}
