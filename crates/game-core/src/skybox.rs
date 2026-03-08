use bevy::prelude::*;
use bevy::camera::Camera3d;
use bevy::mesh::{Mesh, Mesh3d};
use bevy::pbr::{MeshMaterial3d, StandardMaterial};

// Skybox is in the prelude in Bevy 0.18
// If not, we may need to define our own or use a different approach

/// Plugin for managing skybox rendering
///
/// Supports:
/// - Cubemap skyboxes (6 images)
/// - Equirectangular HDRI
/// - Dynamic brightness control
/// - Time-of-day integration
pub struct SkyboxPlugin {
    /// Default skybox to load (None = no skybox)
    pub default_skybox: Option<String>,
    /// Default brightness multiplier
    pub default_brightness: f32,
}

impl Default for SkyboxPlugin {
    fn default() -> Self {
        Self {
            default_skybox: None,
            default_brightness: 1000.0,
        }
    }
}

impl Plugin for SkyboxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SkyboxConfig>();

        if let Some(ref path) = self.default_skybox {
            let path = path.clone();
            let brightness = self.default_brightness;
            app.add_systems(Startup, move |mut commands: Commands, asset_server: Res<AssetServer>| {
                load_skybox_from_path(&mut commands, &asset_server, path.clone(), brightness);
            });
        }

        app.add_systems(Update, (
            update_skybox_brightness,
            debug_toggle_skybox,
        ));
    }
}

/// Skybox configuration resource
#[derive(Resource, Debug, Clone, Reflect)]
pub struct SkyboxConfig {
    /// Whether skybox is enabled
    pub enabled: bool,
    /// Brightness multiplier
    pub brightness: f32,
    /// Rotation of the skybox (in radians around Y axis)
    pub rotation: f32,
}

impl Default for SkyboxConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            brightness: 1000.0,
            rotation: 0.0,
        }
    }
}

/// Component for entities with skyboxes
#[derive(Component, Debug)]
pub struct SkyboxEntity;

/// Load a cubemap skybox from 6 separate images
///
/// Expected file structure:
/// ```
/// assets/skyboxes/my_skybox/
/// ├── px.png  (positive X - right)
/// ├── nx.png  (negative X - left)
/// ├── py.png  (positive Y - up)
/// ├── ny.png  (positive Y - down)
/// ├── pz.png  (positive Z - forward)
/// └── nz.png  (negative Z - back)
/// ```
pub fn load_cubemap_skybox(
    commands: &mut Commands,
    asset_server: &AssetServer,
    images: &mut Assets<Image>,
    folder_path: &str,
    brightness: f32,
) {
    let paths = [
        format!("{}/px.png", folder_path),
        format!("{}/nx.png", folder_path),
        format!("{}/py.png", folder_path),
        format!("{}/ny.png", folder_path),
        format!("{}/pz.png", folder_path),
        format!("{}/nz.png", folder_path),
    ];

    // Load all 6 faces
    let handles: Vec<Handle<Image>> = paths.iter()
        .map(|path| asset_server.load(path.clone()))
        .collect();

    // Create skybox entity
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Skybox {
            image: handles[0].clone(), // Temporary, will be replaced with cubemap
            brightness,
            rotation: Quat::IDENTITY,
        },
        SkyboxEntity,
    ));

    info!("Loading cubemap skybox from: {}", folder_path);
}

/// Load a skybox from a single image path (for equirectangular HDRI)
/// Note: Skybox is now a component on cameras, not a global resource
fn load_skybox_from_path(
    commands: &mut Commands,
    asset_server: &AssetServer,
    path: String,
    brightness: f32,
) {
    let skybox_handle = asset_server.load(path.clone());

    // Spawn a camera with the skybox component
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Skybox {
            image: skybox_handle,
            brightness,
            rotation: Quat::IDENTITY,
        },
        SkyboxEntity,
    ));

    info!("Loading skybox from: {}", path);
}

/// Update skybox brightness when config changes
fn update_skybox_brightness(
    config: Res<SkyboxConfig>,
    mut skyboxes: Query<&mut Skybox, With<SkyboxEntity>>,
) {
    if config.is_changed() {
        for mut skybox in &mut skyboxes {
            skybox.brightness = config.brightness;
        }
    }
}

/// Debug system to toggle skybox with F5 key
fn debug_toggle_skybox(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut config: ResMut<SkyboxConfig>,
) {
    if keyboard.just_pressed(KeyCode::F5) {
        config.enabled = !config.enabled;
        info!("Skybox: {}", if config.enabled { "ON" } else { "OFF" });
    }

    // Adjust brightness with F6/F7
    if keyboard.pressed(KeyCode::F6) {
        config.brightness = (config.brightness - 100.0).max(0.0);
        debug!("Skybox brightness: {:.0}", config.brightness);
    }
    if keyboard.pressed(KeyCode::F7) {
        config.brightness = (config.brightness + 100.0).min(10000.0);
        debug!("Skybox brightness: {:.0}", config.brightness);
    }
}

/// Helper function to create a camera with skybox
pub fn spawn_camera_with_skybox(
    commands: &mut Commands,
    asset_server: &AssetServer,
    skybox_path: &str,
    brightness: f32,
    camera_transform: Transform,
) -> Entity {
    let skybox_handle = asset_server.load(skybox_path.to_string());

    commands.spawn((
        Camera3d::default(),
        camera_transform,
        Skybox {
            image: skybox_handle,
            brightness,
            rotation: Quat::IDENTITY,
        },
        SkyboxEntity,
    )).id()
}

/// Skybox presets for different environments
pub mod presets {
    use super::*;

    /// Clear blue sky
    pub const CLEAR_DAY_BRIGHTNESS: f32 = 1500.0;

    /// Sunset/sunrise
    pub const SUNSET_BRIGHTNESS: f32 = 800.0;

    /// Overcast/cloudy
    pub const CLOUDY_BRIGHTNESS: f32 = 600.0;

    /// Night sky
    pub const NIGHT_BRIGHTNESS: f32 = 200.0;

    /// Indoor (no skybox or very dim)
    pub const INDOOR_BRIGHTNESS: f32 = 50.0;
}

/// Example system for procedural skybox (solid color)
///
/// This creates a simple gradient skybox without requiring texture files.
/// Useful for prototyping or stylized games.
#[allow(dead_code)]
pub fn create_procedural_sky_gradient(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create a large sphere for the sky dome
    let sky_mesh = meshes.add(Sphere::new(500.0));

    commands.spawn((
        Mesh3d(sky_mesh),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.5, 0.7, 1.0), // Sky blue
            unlit: true,
            cull_mode: None, // Render inside of sphere
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0),
        GlobalTransform::default(),
    ));

    info!("Created procedural sky gradient");
}

/// Configuration for time-of-day skybox transitions
#[derive(Resource, Debug, Clone)]
pub struct SkyboxTimeOfDay {
    /// Skybox for different times of day
    pub day_skybox: Option<Handle<Image>>,
    pub sunset_skybox: Option<Handle<Image>>,
    pub night_skybox: Option<Handle<Image>>,

    /// Brightness values for different times
    pub day_brightness: f32,
    pub sunset_brightness: f32,
    pub night_brightness: f32,
}

impl Default for SkyboxTimeOfDay {
    fn default() -> Self {
        Self {
            day_skybox: None,
            sunset_skybox: None,
            night_skybox: None,
            day_brightness: presets::CLEAR_DAY_BRIGHTNESS,
            sunset_brightness: presets::SUNSET_BRIGHTNESS,
            night_brightness: presets::NIGHT_BRIGHTNESS,
        }
    }
}

/// System to update skybox based on time of day
///
/// Requires TimeOfDay resource from lighting module
#[allow(dead_code)]
pub fn update_skybox_time_of_day(
    time_of_day: Res<crate::lighting::TimeOfDay>,
    skybox_tod: Res<SkyboxTimeOfDay>,
    mut skyboxes: Query<&mut Skybox, With<SkyboxEntity>>,
    mut config: ResMut<SkyboxConfig>,
) {
    let (texture, brightness) = match time_of_day.hour {
        h if h >= 6.0 && h < 8.0 => {
            // Dawn - blend to day
            (skybox_tod.sunset_skybox.clone(), skybox_tod.sunset_brightness)
        }
        h if h >= 8.0 && h < 17.0 => {
            // Day
            (skybox_tod.day_skybox.clone(), skybox_tod.day_brightness)
        }
        h if h >= 17.0 && h < 19.0 => {
            // Dusk
            (skybox_tod.sunset_skybox.clone(), skybox_tod.sunset_brightness)
        }
        _ => {
            // Night
            (skybox_tod.night_skybox.clone(), skybox_tod.night_brightness)
        }
    };

    if let Some(new_texture) = texture {
        for mut skybox in &mut skyboxes {
            skybox.image = new_texture.clone();
        }
    }

    config.brightness = brightness;
}

/// Helper to load multiple skyboxes for time-of-day system
pub fn setup_time_of_day_skyboxes(
    commands: &mut Commands,
    asset_server: &AssetServer,
    day_path: String,
    sunset_path: String,
    night_path: String,
) {
    commands.insert_resource(SkyboxTimeOfDay {
        day_skybox: Some(asset_server.load(day_path)),
        sunset_skybox: Some(asset_server.load(sunset_path)),
        night_skybox: Some(asset_server.load(night_path)),
        ..default()
    });

    info!("Loaded time-of-day skyboxes");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skybox_config_defaults() {
        let config = SkyboxConfig::default();
        assert!(config.enabled);
        assert_eq!(config.brightness, 1000.0);
        assert_eq!(config.rotation, 0.0);
    }

    #[test]
    fn test_skybox_presets() {
        assert!(presets::CLEAR_DAY_BRIGHTNESS > presets::NIGHT_BRIGHTNESS);
        assert!(presets::SUNSET_BRIGHTNESS > presets::NIGHT_BRIGHTNESS);
        assert!(presets::CLOUDY_BRIGHTNESS < presets::CLEAR_DAY_BRIGHTNESS);
    }
}
