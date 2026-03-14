use crate::character_rendering::CharacterRenderingPlugin;
use crate::client_config::{GameClientConfig, parse_mouse_button};
use avian3d::prelude::*;
use bevy::{
    color::palettes::css::MAGENTA,
    core_pipeline::Skybox,
    prelude::*,
    render::render_resource::{TextureViewDescriptor, TextureViewDimension},
    window::{CursorGrabMode, CursorOptions},
};
use game_camera::{CameraConfig, CameraPlugin, GameCamera, GameCameraFileConfig};
use game_core::{GameCoreConfig, core_config::parse_key_code, world::WorldAssets};
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

/// Convert a loaded skybox image to a cube texture.
/// Handles equirectangular (2:1), stacked cubemap (1:6), or other stacked layouts.
fn prepare_skybox_cubemap(image: &mut Image) {
    let w = image.width();
    let h = image.height();

    if w == 0 || h == 0 {
        warn!("Skybox image has zero dimensions, skipping cubemap conversion");
        return;
    }

    if h == w * 6 {
        image
            .reinterpret_stacked_2d_as_array(6)
            .expect("Failed to reinterpret stacked cubemap");
    } else if w == h * 2 {
        info!("Converting equirectangular skybox ({}x{}) to cubemap", w, h);
        equirect_to_cubemap(image);
    } else {
        warn!(
            "Skybox image has unexpected aspect ratio ({}x{}), attempting stacked reinterpret",
            w, h
        );
        let layers = h / w;
        if layers > 0 && h == w * layers {
            image
                .reinterpret_stacked_2d_as_array(layers)
                .expect("Failed to reinterpret");
        }
    }

    image.texture_view_descriptor = Some(TextureViewDescriptor {
        dimension: Some(TextureViewDimension::Cube),
        ..default()
    });
}

/// Convert an equirectangular (2:1) image in-place to a stacked cubemap (face_size x face_size*6).
///
/// Uses bilinear interpolation for smooth sampling.
/// Face order: +X, -X, +Y, -Y, +Z, -Z (standard cubemap convention).
fn equirect_to_cubemap(image: &mut Image) {
    let src_w = image.width() as usize;
    let src_h = image.height() as usize;
    let bpp = image
        .texture_descriptor
        .format
        .block_copy_size(None)
        .unwrap_or(16) as usize;
    let src_data = image.data.clone().expect("Skybox image has no data");
    let floats_per_pixel = bpp / 4; // Rgba32Float = 4 floats

    // Use half the equirectangular height as face size, cap at 2048
    let face_size = (src_h / 2).min(2048);
    let out_h = face_size * 6;

    let mut out_data = vec![0u8; face_size * out_h * bpp];

    for (face_idx, face_dirs) in CUBE_FACE_DIRS.iter().enumerate() {
        let y_offset = face_idx * face_size;
        for row in 0..face_size {
            for col in 0..face_size {
                let u = (col as f32 + 0.5) / face_size as f32 * 2.0 - 1.0;
                let v = (row as f32 + 0.5) / face_size as f32 * 2.0 - 1.0;

                let dir = face_dirs(u, v);
                let len = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt();
                let (dx, dy, dz) = (dir[0] / len, dir[1] / len, dir[2] / len);

                let theta = dx.atan2(dz);
                let phi = dy.asin();
                let eq_u = 0.5 + theta / (2.0 * std::f32::consts::PI);
                let eq_v = 0.5 - phi / std::f32::consts::PI;

                // Bilinear interpolation
                let fx = eq_u * src_w as f32 - 0.5;
                let fy = eq_v * src_h as f32 - 0.5;
                let x0 = (fx.floor() as isize).rem_euclid(src_w as isize) as usize;
                let y0 = (fy.floor() as isize).clamp(0, src_h as isize - 1) as usize;
                let x1 = (x0 + 1) % src_w; // wrap horizontally
                let y1 = (y0 + 1).min(src_h - 1);
                let frac_x = fx - fx.floor();
                let frac_y = fy - fy.floor();

                let dst_offset = ((y_offset + row) * face_size + col) * bpp;

                for c in 0..floats_per_pixel {
                    let s00 = read_f32(&src_data, (y0 * src_w + x0) * bpp + c * 4);
                    let s10 = read_f32(&src_data, (y0 * src_w + x1) * bpp + c * 4);
                    let s01 = read_f32(&src_data, (y1 * src_w + x0) * bpp + c * 4);
                    let s11 = read_f32(&src_data, (y1 * src_w + x1) * bpp + c * 4);

                    let top = s00 + (s10 - s00) * frac_x;
                    let bot = s01 + (s11 - s01) * frac_x;
                    let val = top + (bot - top) * frac_y;
                    let bytes = val.to_le_bytes();
                    let o = dst_offset + c * 4;
                    out_data[o..o + 4].copy_from_slice(&bytes);
                }
            }
        }
    }

    image.data = Some(out_data);
    image.texture_descriptor.size.width = face_size as u32;
    image.texture_descriptor.size.height = out_h as u32;
    image
        .reinterpret_stacked_2d_as_array(6)
        .expect("Failed to reinterpret converted cubemap");
}

fn read_f32(data: &[u8], offset: usize) -> f32 {
    f32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ])
}

/// Direction functions for each cube face: +X, -X, +Y, -Y, +Z, -Z.
/// Each takes (u, v) in [-1, 1] and returns [x, y, z].
const CUBE_FACE_DIRS: [fn(f32, f32) -> [f32; 3]; 6] = [
    |u, v| [1.0, -v, -u],  // +X
    |u, v| [-1.0, -v, u],  // -X
    |u, v| [u, 1.0, v],    // +Y
    |u, v| [u, -1.0, -v],  // -Y
    |u, v| [u, -v, 1.0],   // +Z
    |u, v| [-u, -v, -1.0], // -Z
];

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
