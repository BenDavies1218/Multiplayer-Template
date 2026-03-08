use game_core::{
    movement::CROUCH_CAPSULE_HEIGHT,
    protocol::{CharacterMarker, ColorComponent, CrouchState, FloorMarker, ProjectileMarker},
    shared::{CHARACTER_CAPSULE_HEIGHT, CHARACTER_CAPSULE_RADIUS},
};
use game_camera::{CameraConfig, CameraPlugin, GameCamera};
use avian3d::prelude::*;
use bevy::{color::palettes::css::MAGENTA, prelude::*, window::{CursorGrabMode, CursorOptions}};
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

fn init(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, 0.0),
        GameCamera::default(),
    ));

    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
}

fn setup_cursor_grab(
    mut cursor_options: Single<&mut CursorOptions>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    key: Res<ButtonInput<KeyCode>>,
) {
    if key.just_pressed(KeyCode::Escape) {
        cursor_options.visible = true;
        cursor_options.grab_mode = CursorGrabMode::None;
    }
    if mouse_button.just_pressed(MouseButton::Left) {
        cursor_options.visible = false;
        cursor_options.grab_mode = CursorGrabMode::Locked;
    }
}

fn fps_camera_follow(
    mut camera_query: Query<&mut Transform, (With<GameCamera>, Without<CharacterMarker>)>,
    player_query: Query<(&Transform, &CrouchState), (With<CharacterMarker>, With<Predicted>, Without<GameCamera>)>,
) {
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    if let Some((player_transform, crouch_state)) = player_query.iter().next() {
        let capsule_height = if crouch_state.0 { CROUCH_CAPSULE_HEIGHT } else { CHARACTER_CAPSULE_HEIGHT };
        let eye_height = capsule_height / 2.0 + CHARACTER_CAPSULE_RADIUS + 0.5;
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
) {
    for (entity, color) in &character_query {
        commands.entity(entity).insert((
            Mesh3d(meshes.add(Capsule3d::new(
                CHARACTER_CAPSULE_RADIUS,
                CHARACTER_CAPSULE_HEIGHT,
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
) {
    for entity in &projectile_query {
        commands.entity(entity).insert((
            Mesh3d(meshes.add(Sphere::new(1.))),
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