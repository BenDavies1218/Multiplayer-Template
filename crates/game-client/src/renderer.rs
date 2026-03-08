use game_core::{
    protocol::{BlockMarker, CharacterMarker, ColorComponent, FloorMarker, ProjectileMarker},
    shared::{
        BLOCK_HEIGHT, BLOCK_WIDTH, CHARACTER_CAPSULE_HEIGHT, CHARACTER_CAPSULE_RADIUS,
        FLOOR_HEIGHT, FLOOR_WIDTH, WORLD_VISUAL_PATH,
    },
    world::WorldVisual,
};
use avian3d::{math::AsF32, prelude::*};
use bevy::{color::palettes::css::MAGENTA, core_pipeline::Skybox, prelude::*, window::{CursorGrabMode, CursorOptions}, input::mouse::MouseMotion};
use lightyear::prediction::plugin::PredictionSystems;
use lightyear::prediction::rollback::DeterministicPredicted;
use lightyear::prelude::*;
use lightyear_frame_interpolation::{FrameInterpolate, FrameInterpolationPlugin};

/// Marker for the FPS camera
#[derive(Component)]
pub struct FpsCamera {
    pub yaw: f32,
    pub pitch: f32,
    pub sensitivity: f32,
}

impl Default for FpsCamera {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            sensitivity: 0.002,
        }
    }
}

pub struct ExampleRendererPlugin;

impl Plugin for ExampleRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init);

        // Load world from glTF instead of procedural floor:
        app.add_systems(Startup, load_world_visual);

        // Add custom lighting (otherwise uses default from LightingPlugin):
        app.add_systems(Startup, setup_custom_lighting);

        // Uncomment to add skybox:
        // app.add_systems(Startup, setup_skybox);

        app.add_systems(
            Update,
            (
                add_character_cosmetics,
                // Disabled: using world visual instead
                // add_floor_cosmetics,
                add_block_cosmetics,
                fps_camera_follow,
                fps_camera_look,
                setup_cursor_grab,
            ),
        );

        // This is to test a setup where:
        // - enemies are interpolated
        // - they spawn Predicted bullets
        // - we use ReplicateOnce and DisableRollback to stop replicating any packets for these bullets
        app.add_systems(
            PreUpdate,
            add_projectile_cosmetics.before(RollbackSystems::Check),
        );

        // Set up visual interp plugins for Position/Rotation. Position/Rotation is updated in FixedUpdate
        // by the physics plugin so we make sure that in PostUpdate we interpolate it
        app.add_plugins(FrameInterpolationPlugin::<Position>::default());
        app.add_plugins(FrameInterpolationPlugin::<Rotation>::default());

        // Observers that add VisualInterpolationStatus components to entities
        // which receive a Position and are predicted
        app.add_observer(add_visual_interpolation_components);

        // We disable rollbacks for projectiles after the initial rollbacks which brings them to the predicted timeline
        app.add_systems(Last, disable_projectile_rollback);
    }
}

fn init(mut commands: Commands) {
    // Spawn FPS camera (will follow player in fps_camera_follow system)
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, 0.0),
        FpsCamera::default(),
    ));

    // Keep the point light for now (custom lighting will override if enabled)
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
}

/// Set up cursor grab when window is focused
fn setup_cursor_grab(
    mut cursor_options: Single<&mut CursorOptions>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    key: Res<ButtonInput<KeyCode>>,
) {
    // Press Escape to unlock cursor
    if key.just_pressed(KeyCode::Escape) {
        cursor_options.visible = true;
        cursor_options.grab_mode = CursorGrabMode::None;
    }
    // Click to lock cursor
    if mouse_button.just_pressed(MouseButton::Left) {
        cursor_options.visible = false;
        cursor_options.grab_mode = CursorGrabMode::Locked;
    }
}

/// FPS camera look with mouse
fn fps_camera_look(
    mut mouse_motion: MessageReader<MouseMotion>,
    mut camera_query: Query<(&mut FpsCamera, &mut Transform)>,
) {
    let Ok((mut fps_camera, mut transform)) = camera_query.single_mut() else {
        return;
    };

    for motion in mouse_motion.read() {
        fps_camera.yaw -= motion.delta.x * fps_camera.sensitivity;
        fps_camera.pitch -= motion.delta.y * fps_camera.sensitivity;

        // Clamp pitch to prevent camera flipping
        fps_camera.pitch = fps_camera.pitch.clamp(-1.54, 1.54); // ~88 degrees

        // Apply rotation
        transform.rotation = Quat::from_euler(EulerRot::YXZ, fps_camera.yaw, fps_camera.pitch, 0.0);
    }
}

/// Make camera follow the predicted player character
fn fps_camera_follow(
    mut camera_query: Query<&mut Transform, (With<FpsCamera>, Without<CharacterMarker>)>,
    player_query: Query<&Transform, (With<CharacterMarker>, With<Predicted>, Without<FpsCamera>)>,
) {
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    // Follow the predicted player (the one we control)
    if let Some(player_transform) = player_query.iter().next() {
        // Position camera at player's eye level (top of capsule)
        let eye_height = CHARACTER_CAPSULE_HEIGHT / 2.0 + CHARACTER_CAPSULE_RADIUS + 0.5;
        camera_transform.translation = player_transform.translation + Vec3::new(0.0, eye_height, 0.0);
    }
}

/// Add the FrameInterpolate::<Position> component to non-floor entities with
/// component `Position`. Floors don't need to be frame interpolated because we
/// don't expect them to move.
fn add_visual_interpolation_components(
    // We use Position because it's added by avian later, and when it's added
    // we know that Predicted is already present on the entity
    trigger: On<Add, Position>,
    query: Query<Entity, (With<Predicted>, Without<FloorMarker>)>,
    mut commands: Commands,
) {
    if !query.contains(trigger.entity) {
        return;
    }
    commands.entity(trigger.entity).insert((
        FrameInterpolate::<Position> {
            // We must trigger change detection on visual interpolation
            // to make sure that child entities (sprites, meshes, text)
            // are also interpolated
            trigger_change_detection: true,
            ..default()
        },
        FrameInterpolate::<Rotation> {
            // We must trigger change detection on visual interpolation
            // to make sure that child entities (sprites, meshes, text)
            // are also interpolated
            trigger_change_detection: true,
            ..default()
        },
    ));
}

/// Add components to characters that impact how they are rendered. We only
/// want to see the predicted character and not the confirmed character.
fn add_character_cosmetics(
    mut commands: Commands,
    character_query: Query<
        (Entity, &ColorComponent),
        (
            Or<(Added<Predicted>, Added<Replicate>, Added<Interpolated>)>,
            With<CharacterMarker>,
        ),
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, color) in &character_query {
        info!(?entity, "Adding cosmetics to character {:?}", entity);

        // For other players, show the mesh normally
        // For local player (Predicted), we could hide it for true FPS view
        commands.entity(entity).insert((
            Mesh3d(meshes.add(Capsule3d::new(
                CHARACTER_CAPSULE_RADIUS,
                CHARACTER_CAPSULE_HEIGHT,
            ))),
            MeshMaterial3d(materials.add(color.0)),
            // Uncomment to hide local player mesh in FPS mode:
            // Visibility::Hidden,
        ));
    }
}

fn add_projectile_cosmetics(
    mut commands: Commands,
    character_query: Query<
        Entity,
        (
            Or<(Added<Predicted>, Added<Replicate>)>,
            With<ProjectileMarker>,
        ),
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for entity in &character_query {
        info!(?entity, "Adding cosmetics to character {:?}", entity);
        commands.entity(entity).insert((
            Mesh3d(meshes.add(Sphere::new(1.))),
            MeshMaterial3d(materials.add(Color::from(MAGENTA))),
            RigidBody::Dynamic, // needed to add this somewhere, lol
        ));
    }
}

/// Add components to floors that impact how they are rendered. We want to see
/// the replicated floor instead of predicted floors because predicted floors
/// do not exist since floors aren't predicted.
fn add_floor_cosmetics(
    mut commands: Commands,
    floor_query: Query<Entity, Added<FloorMarker>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for entity in &floor_query {
        info!(?entity, "Adding cosmetics to floor {:?}", entity);
        commands.entity(entity).insert((
            Mesh3d(meshes.add(Cuboid::new(FLOOR_WIDTH, FLOOR_HEIGHT, FLOOR_WIDTH))),
            MeshMaterial3d(materials.add(Color::srgb(1.0, 1.0, 1.0))),
        ));
    }
}

/// Add components to blocks that impact how they are rendered. We only want to
/// see the predicted block and not the confirmed block.
fn add_block_cosmetics(
    mut commands: Commands,
    floor_query: Query<Entity, (Or<(Added<Predicted>, Added<Replicate>)>, With<BlockMarker>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for entity in &floor_query {
        info!(?entity, "Adding cosmetics to block {:?}", entity);
        commands.entity(entity).insert((
            Mesh3d(meshes.add(Cuboid::new(BLOCK_WIDTH, BLOCK_HEIGHT, BLOCK_WIDTH))),
            MeshMaterial3d(materials.add(Color::srgb(1.0, 0.0, 1.0))),
        ));
    }
}

fn disable_projectile_rollback(
    mut commands: Commands,
    q_projectile: Query<
        Entity,
        (
            With<Predicted>,
            With<ProjectileMarker>,
            // Or<(With<ProjectileMarker>, With<CharacterMarker>)>,
            // disabling character rollbacks while we debug projectiles with this janky setup

            // We stop checking for state rollbacks after the first frame where the projectile is predicted
            Without<DisableRollback>,
        ),
    >,
) {
    for proj in &q_projectile {
        commands.entity(proj).insert(DisableRollback);
    }
}

/// Example system to load a world visual mesh from glTF
///
/// To use this:
/// 1. Place your world_visual.glb file in assets/models/
/// 2. Uncomment this system in the Plugin::build method above
/// 3. Comment out or remove the add_floor_cosmetics system (or modify it)
///
/// This will load the high-poly visual mesh for rendering.
/// For collision, use the WorldPlugin from game_core::world.
#[allow(dead_code)]
fn load_world_visual(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Load the world visual scene
    let scene_handle = asset_server.load(format!("{}#Scene0", WORLD_VISUAL_PATH));

    commands.spawn((
        SceneRoot(scene_handle),
        WorldVisual,
        Transform::from_xyz(0.0, 0.0, 0.0),
        GlobalTransform::default(),
    ));

    info!("Loading world visual from glTF: {}", WORLD_VISUAL_PATH);
}

/// Example system to set up custom lighting
///
/// This demonstrates how to override the default lighting from LightingPlugin.
/// To use: uncomment the call in Plugin::build above
#[allow(dead_code)]
fn setup_custom_lighting(
    mut commands: Commands,
) {
    use game_core::lighting::*;

    // Option 1: Use a preset
    presets::outdoor_day(&mut commands, ShadowQuality::High);

    // Option 2: Create custom lights
    // create_sun_light(
    //     &mut commands,
    //     Vec3::new(-0.5, -1.0, -0.3).normalize(),
    //     Color::srgb(1.0, 0.95, 0.9),
    //     15000.0,
    //     ShadowQuality::High,
    // );

    // Option 3: Add point lights for torches, lamps, etc.
    // create_point_light_with_shadows(
    //     &mut commands,
    //     Vec3::new(5.0, 2.5, 5.0),
    //     Color::srgb(1.0, 0.7, 0.4), // Warm fire color
    //     1000.0,
    //     15.0,
    //     true, // Enable shadows
    // );

    info!("Custom lighting configured");
}

/// Example system to set up skybox
///
/// This demonstrates how to add a skybox to your scene.
/// To use: uncomment the call in Plugin::build above
#[allow(dead_code)]
fn setup_skybox(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    cameras: Query<Entity, With<Camera3d>>,
) {
    use game_core::shared::*;

    // Option 1: Add skybox to existing camera
    if let Some(camera_entity) = cameras.iter().next() {
        let skybox_handle = asset_server.load(DAY_CLEAR_SKYBOX);
        commands.entity(camera_entity).insert(Skybox {
            image: skybox_handle,
            brightness: 1500.0,
            rotation: Quat::IDENTITY,
        });
        info!("Skybox added to camera");
    }

    // Option 2: Time-of-day transitions between skyboxes
    // use game_core::skybox::*;
    // setup_time_of_day_skyboxes(
    //     &mut commands,
    //     &asset_server,
    //     DAY_CLEAR_SKYBOX.to_string(),
    //     SUNSET_SKY_SKYBOX.to_string(),
    //     EVENING_SKY_SKYBOX.to_string(),
    // );
}
