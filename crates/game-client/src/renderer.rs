use game_core::{
    networking::protocol::{CharacterMarker, ColorComponent, CrouchState, FloorMarker, ProjectileMarker},
    core_config::parse_key_code,
    GameCoreConfig,
};
use game_camera::{CameraConfig, CameraPlugin, GameCamera};
use avian3d::prelude::*;
use bevy::{color::palettes::css::MAGENTA, prelude::*, window::{CursorGrabMode, CursorOptions}};
use lightyear::prelude::*;
use lightyear_frame_interpolation::{FrameInterpolate, FrameInterpolationPlugin};
use crate::client_config::{GameClientConfig, parse_mouse_button};

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

impl Plugin for FirstPersonPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CameraPlugin {
            config: self.camera_config.clone(),
        });

        app.add_systems(Startup, init);
        app.add_systems(Update, (add_character_cosmetics, fps_camera_follow, setup_cursor_grab));
        app.add_systems(PreUpdate, add_projectile_cosmetics.before(RollbackSystems::Check));

        // Frame interpolation for smooth rendering
        app.add_plugins(FrameInterpolationPlugin::<Position>::default());
        app.add_plugins(FrameInterpolationPlugin::<Rotation>::default());
        app.add_observer(add_visual_interpolation_components);

        app.add_systems(Last, disable_projectile_rollback);
    }
}

fn init(mut commands: Commands, config: Res<GameClientConfig>) {
    let pos = config.rendering.camera_start_position;
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(pos[0], pos[1], pos[2]),
        GameCamera::default(),
    ));
}

fn setup_cursor_grab(
    mut cursor_options: Single<&mut CursorOptions>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    key: Res<ButtonInput<KeyCode>>,
    config: Res<GameClientConfig>,
) {
    let release_key = parse_key_code(&config.input.cursor_release_key).unwrap_or(KeyCode::Escape);
    let grab_button = parse_mouse_button(&config.input.cursor_grab_button).unwrap_or(MouseButton::Left);
    if key.just_pressed(release_key) {
        cursor_options.visible = true;
        cursor_options.grab_mode = CursorGrabMode::None;
    }
    if mouse_button.just_pressed(grab_button) {
        cursor_options.visible = false;
        cursor_options.grab_mode = CursorGrabMode::Locked;
    }
}

fn fps_camera_follow(
    mut camera_query: Query<&mut Transform, (With<GameCamera>, Without<CharacterMarker>)>,
    player_query: Query<(&Transform, &CrouchState), (With<CharacterMarker>, With<Predicted>, Without<GameCamera>)>,
    core_config: Res<GameCoreConfig>,
    client_config: Res<GameClientConfig>,
) {
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    if let Some((player_transform, crouch_state)) = player_query.iter().next() {
        let capsule_height = if crouch_state.0 { core_config.movement.crouch_capsule_height } else { core_config.character.capsule_height };
        let eye_height = capsule_height / 2.0 + core_config.character.capsule_radius + client_config.rendering.eye_height_offset;
        camera_transform.translation = player_transform.translation + Vec3::new(0.0, eye_height, 0.0);
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
    core_config: Res<GameCoreConfig>,
) {
    for (entity, color) in &character_query {
        commands.entity(entity).insert((
            Mesh3d(meshes.add(Capsule3d::new(
                core_config.character.capsule_radius,
                core_config.character.capsule_height,
            ))),
            MeshMaterial3d(materials.add(color.0)),
        ));
    }
}

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
) {
    for entity in &projectile_query {
        commands.entity(entity).insert((
            Mesh3d(meshes.add(Sphere::new(config.rendering.projectile_radius))),
            MeshMaterial3d(materials.add(Color::from(MAGENTA))),
            RigidBody::Dynamic,
        ));
    }
}

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