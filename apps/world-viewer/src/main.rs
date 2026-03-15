//! Standalone World Viewer
//!
//! A simple app for testing world rendering without any networking.
//! This loads your world visual and collision meshes for quick iteration.

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use game_camera::{CameraConfig, CameraPlugin, CameraViewMode, GameCamera, GameCameraFileConfig};
use game_core::core_config::{DebugColorsConfig, DebugToggleKeysConfig};
use game_core::performance_config::GamePerformanceConfig;
use game_core::simulation_config::GameSimulationConfig;
use game_core::skybox::SkyboxPlugin;
use game_core::utils::config_hot_reload::{ConfigHotReloadPlugin, ConfigWatchExt};
use game_core::utils::config_loader::load_config;
use game_core::world_config::GameWorldConfig;

fn main() {
    let simulation_config: GameSimulationConfig = load_config("game_simulation_config.json");
    let performance_config: GamePerformanceConfig = load_config("game_performance_config.json");
    let world_config: GameWorldConfig = load_config("game_world_config.json");
    let camera_config: GameCameraFileConfig = load_config("game_camera_config.json");

    let mut app = App::new();
    app.insert_resource(simulation_config.clone());
    app.insert_resource(world_config.clone());
    // Insert debug config resources as standalone resources for game-core debug systems
    app.insert_resource(DebugColorsConfig::default());
    app.insert_resource(DebugToggleKeysConfig::default());
    app.add_plugins(
        DefaultPlugins
            .set(AssetPlugin {
                file_path: game_core::utils::config_loader::resolve_asset_path_for_bevy(),
                meta_check: bevy::asset::AssetMetaCheck::Never,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "World Viewer - Test Your World Assets".to_string(),
                    resolution: WindowResolution::new(1920, 1080),
                    ..default()
                }),
                ..default()
            }),
    );
    app.add_plugins(ConfigHotReloadPlugin::default());
    app.watch_config::<GameSimulationConfig>("game_simulation_config.json");
    app.watch_config::<GamePerformanceConfig>("game_performance_config.json");
    app.watch_config::<GameWorldConfig>("game_world_config.json");
    app.watch_config::<GameCameraFileConfig>("game_camera_config.json");
    app.add_plugins(CameraPlugin {
        config: CameraConfig::free_view_from_config(&camera_config),
    });
    app.add_plugins(PhysicsPlugins::default());
    app.add_plugins(game_core::world::WorldPlugin {
        config: game_core::world::WorldPluginConfig::viewer(),
    });
    app.add_plugins(game_core::zones::ZonePlugin {
        config: game_core::zones::ZonePluginConfig::viewer(),
    });
    app.add_plugins(game_dynamic::DynamicPlugin {
        config: game_dynamic::DynamicPluginConfig::viewer(),
    });
    app.add_plugins(SkyboxPlugin);

    if performance_config.enable_diagnostics {
        app.add_plugins(game_diagnostics::DiagnosticsPlugin::viewer());
    }

    app.add_systems(Startup, setup);
    app.add_systems(
        Update,
        (
            toggle_camera_mode,
            viewer_camera_controller,
            cursor_grab,
            auto_play_gltf_animations,
        ),
    );
    app.run();
}

/// Marker for the test capsule that camera follows in first/third person.
#[derive(Component)]
struct ViewerTarget;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    sim_config: Res<GameSimulationConfig>,
    world_config: Res<GameWorldConfig>,
) {
    info!("=== World Viewer Started ===");
    info!("Controls:");
    info!("  WASD - Move (camera in FreeView, capsule in FP/TP)");
    info!("  Space/Shift - Up/Down");
    info!("  Ctrl - Speed boost");
    info!("  Mouse - Look around");
    info!("  Click - Grab cursor");
    info!("  Escape - Release cursor");
    info!("  V - Toggle camera mode (FreeView / FirstPerson / ThirdPerson)");
    info!("  C - Toggle collision mesh visualization");
    info!("  D - Toggle dynamic object visualization");
    info!("  1 - Toggle red light (intensity + color)");
    info!("  2 - Toggle blue light (intensity + color)");
    info!("");
    info!(
        "Loading world from: {}",
        world_config.world_assets.visual_path
    );

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        GameCamera::default(),
    ));

    // Spawn a test character capsule with physics to test collision
    let capsule_mesh = Capsule3d::new(
        sim_config.character.capsule_radius,
        sim_config.character.capsule_height,
    );
    commands.spawn((
        Name::new("Test Character (Physics Enabled)"),
        ViewerTarget,
        Mesh3d(asset_server.add(Mesh::from(capsule_mesh))),
        MeshMaterial3d(asset_server.add(StandardMaterial {
            base_color: Color::srgb(0.0, 1.0, 0.0),
            ..default()
        })),
        Transform::from_xyz(0.0, 5.0, 0.0),
        RigidBody::Dynamic,
        Collider::capsule(
            sim_config.character.capsule_radius,
            sim_config.character.capsule_height,
        ),
        LockedAxes::default()
            .lock_rotation_x()
            .lock_rotation_y()
            .lock_rotation_z(),
    ));

    info!("Test capsule spawned at Y=5.0 - should fall and land on collision mesh");
    info!("Test lights spawned: 'test_light_red' at (-3,3,0), 'test_light_blue' at (3,3,0)");
    info!("World viewer setup complete!");
}

/// Toggle camera mode with V key: FreeView -> FirstPerson -> ThirdPerson.
fn toggle_camera_mode(
    keys: Res<ButtonInput<KeyCode>>,
    mut config: ResMut<CameraConfig>,
    file_config: Res<GameCameraFileConfig>,
) {
    if !keys.just_pressed(KeyCode::KeyV) {
        return;
    }

    let new_config = match config.view_mode {
        CameraViewMode::FreeView => {
            info!("[CAMERA] Switched to FirstPerson");
            CameraConfig::first_person_from_config(&file_config)
        }
        CameraViewMode::FirstPerson => {
            info!("[CAMERA] Switched to ThirdPerson");
            CameraConfig::third_person_from_config(&file_config)
        }
        CameraViewMode::ThirdPerson => {
            info!("[CAMERA] Switched to FreeView");
            CameraConfig::free_view_from_config(&file_config)
        }
    };
    *config = new_config;
}

/// Unified camera controller that behaves differently per mode.
fn viewer_camera_controller(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<CameraConfig>,
    sim_config: Res<GameSimulationConfig>,
    mut camera_query: Query<(&GameCamera, &mut Transform), Without<ViewerTarget>>,
    mut target_query: Query<(&Transform, &mut LinearVelocity), With<ViewerTarget>>,
) {
    let Ok((game_camera, mut cam_transform)) = camera_query.single_mut() else {
        return;
    };

    match config.view_mode {
        CameraViewMode::FreeView => {
            let mut direction = Vec3::ZERO;

            if keys.pressed(KeyCode::KeyW) {
                direction += cam_transform.forward().as_vec3();
            }
            if keys.pressed(KeyCode::KeyS) {
                direction -= cam_transform.forward().as_vec3();
            }
            if keys.pressed(KeyCode::KeyA) {
                direction -= cam_transform.right().as_vec3();
            }
            if keys.pressed(KeyCode::KeyD) {
                direction += cam_transform.right().as_vec3();
            }
            if keys.pressed(KeyCode::Space) {
                direction += Vec3::Y;
            }
            if keys.pressed(KeyCode::ShiftLeft) {
                direction -= Vec3::Y;
            }

            let speed = if keys.pressed(KeyCode::ControlLeft) {
                config.free_camera_speed * 3.0
            } else {
                config.free_camera_speed
            };

            if direction.length() > 0.0 {
                direction = direction.normalize();
                cam_transform.translation += direction * speed * time.delta_secs();
            }
        }
        CameraViewMode::FirstPerson | CameraViewMode::ThirdPerson => {
            let Ok((target_transform, mut velocity)) = target_query.single_mut() else {
                return;
            };

            let forward = game_camera.forward_direction();
            let right = game_camera.right_direction();
            let mut move_dir = Vec3::ZERO;

            if keys.pressed(KeyCode::KeyW) {
                move_dir += forward;
            }
            if keys.pressed(KeyCode::KeyS) {
                move_dir -= forward;
            }
            if keys.pressed(KeyCode::KeyA) {
                move_dir -= right;
            }
            if keys.pressed(KeyCode::KeyD) {
                move_dir += right;
            }

            let speed = sim_config.movement.max_speed;
            if move_dir.length() > 0.0 {
                move_dir = move_dir.normalize();
                velocity.x = move_dir.x * speed;
                velocity.z = move_dir.z * speed;
            } else {
                velocity.x = 0.0;
                velocity.z = 0.0;
            }

            if keys.just_pressed(KeyCode::Space) {
                velocity.y = sim_config.movement.jump_impulse;
            }

            let target_pos = target_transform.translation;
            let eye_height = sim_config.character.capsule_height / 2.0
                + sim_config.character.capsule_radius;

            match config.view_mode {
                CameraViewMode::FirstPerson => {
                    cam_transform.translation = target_pos + Vec3::new(0.0, eye_height, 0.0);
                }
                CameraViewMode::ThirdPerson => {
                    let look_dir =
                        Quat::from_euler(EulerRot::YXZ, game_camera.yaw, game_camera.pitch, 0.0)
                            * Vec3::NEG_Z;
                    let offset = target_pos + Vec3::new(0.0, config.third_person_height, 0.0)
                        - look_dir * config.third_person_distance;
                    if config.smooth_camera {
                        cam_transform.translation =
                            cam_transform.translation.lerp(offset, config.smooth_factor);
                    } else {
                        cam_transform.translation = offset;
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}

/// Handle cursor grab/release.
fn cursor_grab(
    keys: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut cursor_options: Single<&mut bevy::window::CursorOptions>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        cursor_options.visible = true;
        cursor_options.grab_mode = bevy::window::CursorGrabMode::None;
    }
    if mouse_button.just_pressed(MouseButton::Left) {
        cursor_options.visible = false;
        cursor_options.grab_mode = bevy::window::CursorGrabMode::Locked;
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
