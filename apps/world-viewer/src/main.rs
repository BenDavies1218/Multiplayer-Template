//! Standalone World Viewer
//!
//! A simple app for testing world rendering without any networking.
//! This loads your world visual and collision meshes for quick iteration.

use bevy::prelude::*;
use bevy::window::WindowResolution;
use avian3d::prelude::*;
use game_core::shared::{WORLD_VISUAL_PATH, CHARACTER_CAPSULE_HEIGHT, CHARACTER_CAPSULE_RADIUS};

fn main() {
    // Set asset root to workspace root (not package directory)
    // This allows world-viewer to find assets/ folder in the workspace root
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.parent().unwrap().parent().unwrap();
    unsafe {
        std::env::set_var("BEVY_ASSET_ROOT", workspace_root);
    }

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "World Viewer - Test Your World Assets".to_string(),
                resolution: WindowResolution::new(1920, 1080),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(game_core::world::WorldPlugin {
            config: game_core::world::WorldPluginConfig::viewer(),
        })
        .add_systems(Startup, setup)
        .add_systems(Update, camera_controller)
        .run();
}

#[derive(Component)]
struct FlyCamera {
    pub yaw: f32,
    pub pitch: f32,
    pub speed: f32,
    pub sensitivity: f32,
}

impl Default for FlyCamera {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            speed: 10.0,
            sensitivity: 0.002,
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    info!("=== World Viewer Started ===");
    info!("Controls:");
    info!("  WASD - Move camera");
    info!("  Space/Shift - Up/Down");
    info!("  Ctrl - Speed boost");
    info!("  Mouse - Look around");
    info!("  Click - Grab cursor");
    info!("  Escape - Release cursor");
    info!("  C - Toggle collision mesh visualization");
    info!("");
    info!("Loading world from: {}", WORLD_VISUAL_PATH);

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        FlyCamera::default(),
    ));

    // Spawn a test character capsule with physics to test collision
    // This capsule will fall due to gravity and collide with the world
    let capsule_mesh = Capsule3d::new(CHARACTER_CAPSULE_RADIUS, CHARACTER_CAPSULE_HEIGHT);
    commands.spawn((
        Name::new("Test Character (Physics Enabled)"),
        Mesh3d(asset_server.add(Mesh::from(capsule_mesh))),
        MeshMaterial3d(asset_server.add(StandardMaterial {
            base_color: Color::srgb(0.0, 1.0, 0.0),
            ..default()
        })),
        Transform::from_xyz(0.0, 5.0, 0.0), // Spawn higher to see it fall
        // Physics components
        RigidBody::Dynamic,
        Collider::capsule(CHARACTER_CAPSULE_RADIUS, CHARACTER_CAPSULE_HEIGHT),
        // Lock rotation so it stays upright
        LockedAxes::default()
            .lock_rotation_x()
            .lock_rotation_y()
            .lock_rotation_z(),
    ));

    info!("Test capsule spawned at Y=5.0 - should fall and land on collision mesh");
    info!("World viewer setup complete!");
}

/// Free-fly camera controller
fn camera_controller(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: MessageReader<bevy::input::mouse::MouseMotion>,
    mut cursor_options: Single<&mut bevy::window::CursorOptions>,
    mut camera_query: Query<(&mut Transform, &mut FlyCamera)>,
) {
    let Ok((mut transform, mut fly_camera)) = camera_query.single_mut() else {
        return;
    };

    // Cursor grab/release
    if keys.just_pressed(KeyCode::Escape) {
        cursor_options.visible = true;
        cursor_options.grab_mode = bevy::window::CursorGrabMode::None;
    }
    if mouse_button.just_pressed(MouseButton::Left) {
        cursor_options.visible = false;
        cursor_options.grab_mode = bevy::window::CursorGrabMode::Locked;
    }

    // Mouse look
    for motion in mouse_motion.read() {
        fly_camera.yaw -= motion.delta.x * fly_camera.sensitivity;
        fly_camera.pitch -= motion.delta.y * fly_camera.sensitivity;
        fly_camera.pitch = fly_camera.pitch.clamp(-1.54, 1.54);
        transform.rotation = Quat::from_euler(EulerRot::YXZ, fly_camera.yaw, fly_camera.pitch, 0.0);
    }

    // Movement
    let mut direction = Vec3::ZERO;

    if keys.pressed(KeyCode::KeyW) {
        direction += transform.forward().as_vec3();
    }
    if keys.pressed(KeyCode::KeyS) {
        direction -= transform.forward().as_vec3();
    }
    if keys.pressed(KeyCode::KeyA) {
        direction -= transform.right().as_vec3();
    }
    if keys.pressed(KeyCode::KeyD) {
        direction += transform.right().as_vec3();
    }
    if keys.pressed(KeyCode::Space) {
        direction += Vec3::Y;
    }
    if keys.pressed(KeyCode::ShiftLeft) {
        direction -= Vec3::Y;
    }

    // Speed boost with Ctrl
    let speed = if keys.pressed(KeyCode::ControlLeft) {
        fly_camera.speed * 3.0
    } else {
        fly_camera.speed
    };

    if direction.length() > 0.0 {
        direction = direction.normalize();
        transform.translation += direction * speed * time.delta_secs();
    }
}

