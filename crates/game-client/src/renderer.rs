use crate::character_rendering::CharacterRenderingPlugin;
use crate::client_config::{GameClientConfig, parse_mouse_button};
use avian3d::prelude::*;
use bevy::{
    color::palettes::css::MAGENTA,
    core_pipeline::Skybox,
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};
use game_camera::{CameraConfig, CameraPlugin, GameCamera, GameCameraFileConfig};
use game_core::{
    GameCoreConfig, core_config::parse_key_code, skybox::prepare_skybox_cubemap, world::WorldAssets,
};
use game_networking::protocol::{CharacterMarker, CrouchState, FloorMarker, ProjectileMarker};
use lightyear::prelude::*;
use lightyear_frame_interpolation::{FrameInterpolate, FrameInterpolationPlugin};

pub struct FirstPersonPlugin {
    pub camera_config: CameraConfig,
}

impl Default for FirstPersonPlugin {
    fn default() -> Self {
        Self {
            camera_config: CameraConfig::first_person(),
        }
    }
}

/// Tracks all client assets that must load before the camera spawns.
#[derive(Resource)]
struct PendingClientAssets {
    skybox: Handle<Image>,
    ready: bool,
}

impl Plugin for FirstPersonPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CameraPlugin {
            config: self.camera_config.clone(),
        });

        app.add_plugins(CharacterRenderingPlugin);

        app.add_systems(Startup, start_loading_assets);
        app.add_systems(
            Update,
            (
                check_assets_loaded,
                fps_camera_follow,
                setup_cursor_grab,
                auto_play_gltf_animations,
            ),
        );
        app.add_systems(
            PreUpdate,
            add_projectile_cosmetics.before(RollbackSystems::Check),
        );

        // Frame interpolation for smooth rendering
        app.add_plugins(FrameInterpolationPlugin::<Position>::default());
        app.add_plugins(FrameInterpolationPlugin::<Rotation>::default());
        app.add_observer(add_visual_interpolation_components);

        app.add_systems(Last, disable_projectile_rollback);
    }
}

/// Kick off loading the skybox. World assets are already loaded by WorldPlugin in Startup.
fn start_loading_assets(
    mut commands: Commands,
    core_config: Res<GameCoreConfig>,
    asset_server: Res<AssetServer>,
) {
    let skybox = asset_server.load(&core_config.world_assets.skybox_path);
    commands.insert_resource(PendingClientAssets {
        skybox,
        ready: false,
    });
}

/// Wait for ALL client assets (world visual, collision, skybox) to finish loading,
/// then spawn the camera with the skybox attached.
fn check_assets_loaded(
    mut commands: Commands,
    mut pending: Option<ResMut<PendingClientAssets>>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    world_assets: Option<Res<WorldAssets>>,
    camera_file_config: Res<GameCameraFileConfig>,
    camera_query: Query<Entity, With<GameCamera>>,
) {
    let Some(ref mut pending) = pending else {
        return;
    };
    if pending.ready {
        return;
    }

    // Check world assets
    if let Some(ref world) = world_assets
        && let Some(ref visual) = world.visual
        && !asset_server.is_loaded_with_dependencies(visual)
    {
        return;
    }

    // Check skybox
    if !asset_server.is_loaded_with_dependencies(&pending.skybox) {
        return;
    }

    // --- All assets loaded ---
    info!("All client assets loaded, spawning camera");
    pending.ready = true;

    // Convert skybox to cubemap
    let skybox_handle = pending.skybox.clone();
    if let Some(image) = images.get_mut(&skybox_handle) {
        prepare_skybox_cubemap(image);
    }

    // Spawn camera if not already present
    if camera_query.is_empty() {
        let pos = camera_file_config.start_position;
        commands.spawn((
            Camera3d::default(),
            Transform::from_xyz(pos[0], pos[1], pos[2]),
            GameCamera::default(),
            Skybox {
                image: skybox_handle,
                brightness: 100.0,
                rotation: Quat::IDENTITY,
            },
        ));
    }
}

/// Auto-play all animations from loaded glTF scenes on loop.
fn auto_play_gltf_animations(
    mut players: Query<(&mut AnimationPlayer, &AnimationGraphHandle), Added<AnimationPlayer>>,
    graphs: Res<Assets<AnimationGraph>>,
) {
    for (mut player, graph_handle) in &mut players {
        let Some(graph) = graphs.get(&graph_handle.0) else {
            continue;
        };
        for index in graph.nodes() {
            player.play(index).repeat();
        }
    }
}

fn setup_cursor_grab(
    mut cursor_options: Single<&mut CursorOptions>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    key: Res<ButtonInput<KeyCode>>,
    config: Res<GameClientConfig>,
) {
    let release_key = parse_key_code(&config.input.cursor_release_key).unwrap_or(KeyCode::Escape);
    let grab_button =
        parse_mouse_button(&config.input.cursor_grab_button).unwrap_or(MouseButton::Left);
    if key.just_pressed(release_key) {
        cursor_options.visible = true;
        cursor_options.grab_mode = CursorGrabMode::None;
    }
    if mouse_button.just_pressed(grab_button) {
        cursor_options.visible = false;
        cursor_options.grab_mode = CursorGrabMode::Locked;
    }
}

#[allow(clippy::type_complexity)]
fn fps_camera_follow(
    mut camera_query: Query<&mut Transform, (With<GameCamera>, Without<CharacterMarker>)>,
    player_query: Query<
        (&Transform, &CrouchState),
        (With<CharacterMarker>, With<Predicted>, Without<GameCamera>),
    >,
    core_config: Res<GameCoreConfig>,
    client_config: Res<GameClientConfig>,
) {
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    if let Some((player_transform, crouch_state)) = player_query.iter().next() {
        let capsule_height = if crouch_state.0 {
            core_config.movement.crouch_capsule_height
        } else {
            core_config.character.capsule_height
        };
        let eye_height = capsule_height / 2.0
            + core_config.character.capsule_radius
            + client_config.rendering.eye_height_offset;
        camera_transform.translation =
            player_transform.translation + Vec3::new(0.0, eye_height, 0.0);
    }
}

fn add_visual_interpolation_components(
    trigger: On<Add, Position>,
    query: Query<Entity, (With<Predicted>, Without<FloorMarker>)>,
    mut commands: Commands,
) {
    if !query.contains(trigger.entity) {
        return;
    }
    commands.entity(trigger.entity).insert((
        FrameInterpolate::<Position> {
            trigger_change_detection: true,
            ..default()
        },
        FrameInterpolate::<Rotation> {
            trigger_change_detection: true,
            ..default()
        },
    ));
}

#[allow(clippy::type_complexity)]
fn add_projectile_cosmetics(
    mut commands: Commands,
    projectile_query: Query<
        Entity,
        (
            Or<(Added<Predicted>, Added<Replicate>)>,
            With<ProjectileMarker>,
        ),
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<GameClientConfig>,
    core_config: Res<GameCoreConfig>,
) {
    for entity in &projectile_query {
        commands.entity(entity).insert((
            Mesh3d(meshes.add(Sphere::new(config.rendering.projectile_radius))),
            MeshMaterial3d(materials.add(Color::from(MAGENTA))),
            RigidBody::Dynamic,
            Collider::sphere(core_config.projectile.radius),
        ));
    }
}

#[allow(clippy::type_complexity)]
fn disable_projectile_rollback(
    mut commands: Commands,
    projectile_query: Query<
        Entity,
        (
            With<Predicted>,
            With<ProjectileMarker>,
            Without<DisableRollback>,
        ),
    >,
) {
    for entity in &projectile_query {
        commands.entity(entity).insert(DisableRollback);
    }
}
