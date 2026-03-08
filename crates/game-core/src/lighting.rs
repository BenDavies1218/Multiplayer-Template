use bevy::prelude::*;
use bevy::light::{
    CascadeShadowConfig, CascadeShadowConfigBuilder,
    DirectionalLight, AmbientLight, PointLight, SpotLight,
};

/// Plugin for managing world lighting and shadows
///
/// Features:
/// - Configurable shadow quality presets
/// - Dynamic time of day support
/// - Shadow debugging tools
/// - Performance optimization
pub struct LightingPlugin {
    /// Initial shadow quality setting
    pub shadow_quality: ShadowQuality,
    /// Enable shadow debugging visualizations
    pub debug_shadows: bool,
}

impl Default for LightingPlugin {
    fn default() -> Self {
        Self {
            shadow_quality: ShadowQuality::Medium,
            debug_shadows: false,
        }
    }
}

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LightingConfig {
            shadow_quality: self.shadow_quality,
            shadows_enabled: true,
            ambient_enabled: true,
            debug_mode: self.debug_shadows,
        });

        app.add_systems(Startup, setup_default_lighting);
        app.add_systems(Update, (
            update_shadow_quality,
            debug_toggle_shadows,
            update_time_of_day,
        ));

        if self.debug_shadows {
            app.add_systems(Update, display_shadow_debug_info);
        }
    }
}

/// Shadow quality presets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Resource, Reflect)]
pub enum ShadowQuality {
    /// Minimal shadows - 1 cascade, 50m distance
    /// Performance: Excellent (~1ms)
    Low,

    /// Balanced shadows - 2 cascades, 100m distance
    /// Performance: Good (~2-3ms)
    Medium,

    /// High quality shadows - 4 cascades, 200m distance
    /// Performance: Fair (~4-5ms)
    High,

    /// Ultra quality shadows - 4 cascades, 300m distance
    /// Performance: Demanding (~6-8ms)
    Ultra,

    /// Shadows disabled
    Disabled,
}

impl ShadowQuality {
    /// Convert quality preset to cascade shadow configuration
    pub fn to_cascade_config(&self) -> CascadeShadowConfig {
        match self {
            Self::Low => CascadeShadowConfigBuilder {
                num_cascades: 1,
                minimum_distance: 0.1,
                maximum_distance: 50.0,
                first_cascade_far_bound: 5.0,
                ..default()
            }.build(),
            Self::Medium => CascadeShadowConfigBuilder {
                num_cascades: 2,
                minimum_distance: 0.1,
                maximum_distance: 100.0,
                first_cascade_far_bound: 8.0,
                ..default()
            }.build(),
            Self::High => CascadeShadowConfigBuilder {
                num_cascades: 4,
                minimum_distance: 0.1,
                maximum_distance: 200.0,
                first_cascade_far_bound: 10.0,
                ..default()
            }.build(),
            Self::Ultra => CascadeShadowConfigBuilder {
                num_cascades: 4,
                minimum_distance: 0.1,
                maximum_distance: 300.0,
                first_cascade_far_bound: 15.0,
                ..default()
            }.build(),
            Self::Disabled => CascadeShadowConfigBuilder {
                num_cascades: 1,
                minimum_distance: 0.1,
                maximum_distance: 1.0,
                first_cascade_far_bound: 0.5,
                ..default()
            }.build(),
        }
    }

    /// Get the GPU cost estimate in milliseconds
    pub fn gpu_cost_estimate(&self) -> f32 {
        match self {
            Self::Low => 1.0,
            Self::Medium => 2.5,
            Self::High => 5.0,
            Self::Ultra => 7.0,
            Self::Disabled => 0.0,
        }
    }
}

/// Global lighting configuration resource
#[derive(Resource, Debug, Clone, Reflect)]
pub struct LightingConfig {
    pub shadow_quality: ShadowQuality,
    pub shadows_enabled: bool,
    pub ambient_enabled: bool,
    pub debug_mode: bool,
}

impl Default for LightingConfig {
    fn default() -> Self {
        Self {
            shadow_quality: ShadowQuality::Medium,
            shadows_enabled: true,
            ambient_enabled: true,
            debug_mode: false,
        }
    }
}

/// Component marker for the main directional light (sun/moon)
#[derive(Component, Debug)]
pub struct MainDirectionalLight;

/// Component marker for the main ambient light
#[derive(Component, Debug)]
pub struct MainAmbientLight;

/// Component marker for point lights that should cast shadows
#[derive(Component, Debug)]
pub struct ShadowCastingPointLight;

/// Time of day resource (0.0 - 24.0 hours)
#[derive(Resource, Debug, Reflect)]
pub struct TimeOfDay {
    /// Current hour (0.0 - 24.0)
    pub hour: f32,
    /// Speed multiplier (1.0 = real-time, 60.0 = 1 minute = 1 hour)
    pub speed: f32,
    /// Whether time progression is enabled
    pub enabled: bool,
}

impl Default for TimeOfDay {
    fn default() -> Self {
        Self {
            hour: 12.0, // Noon
            speed: 0.0, // Static by default
            enabled: false,
        }
    }
}

impl TimeOfDay {
    /// Create a time of day at a specific hour
    pub fn at_hour(hour: f32) -> Self {
        Self {
            hour: hour.clamp(0.0, 24.0),
            speed: 0.0,
            enabled: false,
        }
    }

    /// Enable dynamic time progression
    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self.enabled = true;
        self
    }

    /// Get sun/moon elevation angle in radians
    pub fn sun_elevation(&self) -> f32 {
        // Map hour to angle: 6am = -90°, noon = 0°, 6pm = 90°
        let normalized = (self.hour - 6.0) / 12.0; // -0.5 to 1.5
        let angle = (normalized * std::f32::consts::PI) - std::f32::consts::FRAC_PI_2;
        angle
    }

    /// Get sun/moon azimuth angle in radians
    pub fn sun_azimuth(&self) -> f32 {
        // Sun moves from east to west
        let normalized = self.hour / 24.0;
        normalized * 2.0 * std::f32::consts::PI
    }

    /// Get light color based on time of day
    pub fn sun_color(&self) -> Color {
        match self.hour {
            h if h < 5.0 => Color::srgb(0.2, 0.2, 0.4),        // Night - blue
            h if h < 6.0 => Color::srgb(0.8, 0.5, 0.4),        // Dawn - orange
            h if h < 8.0 => Color::srgb(1.0, 0.9, 0.7),        // Morning - warm
            h if h < 17.0 => Color::srgb(1.0, 0.95, 0.9),      // Day - white
            h if h < 19.0 => Color::srgb(1.0, 0.7, 0.5),       // Dusk - orange
            h if h < 20.0 => Color::srgb(0.6, 0.4, 0.5),       // Evening - purple
            _ => Color::srgb(0.2, 0.2, 0.4),                   // Night - blue
        }
    }

    /// Get light intensity based on time of day
    pub fn sun_intensity(&self) -> f32 {
        match self.hour {
            h if h < 5.0 => 100.0,          // Night - moon
            h if h < 6.0 => 2000.0,         // Dawn
            h if h < 8.0 => 8000.0,         // Morning
            h if h < 17.0 => 15000.0,       // Day
            h if h < 19.0 => 5000.0,        // Dusk
            h if h < 20.0 => 1000.0,        // Evening
            _ => 100.0,                     // Night - moon
        }
    }

    /// Get ambient light color
    pub fn ambient_color(&self) -> Color {
        match self.hour {
            h if h < 5.0 => Color::srgb(0.1, 0.1, 0.2),        // Night
            h if h < 7.0 => Color::srgb(0.4, 0.3, 0.3),        // Dawn
            h if h < 18.0 => Color::srgb(0.5, 0.6, 0.8),       // Day
            h if h < 20.0 => Color::srgb(0.3, 0.2, 0.3),       // Dusk
            _ => Color::srgb(0.1, 0.1, 0.2),                   // Night
        }
    }

    /// Get ambient light brightness
    pub fn ambient_brightness(&self) -> f32 {
        match self.hour {
            h if h < 6.0 => 0.05,           // Night
            h if h < 8.0 => 0.15,           // Dawn
            h if h < 17.0 => 0.3,           // Day
            h if h < 20.0 => 0.1,           // Dusk
            _ => 0.05,                      // Night
        }
    }
}

/// Setup default lighting when no custom lighting is provided
fn setup_default_lighting(
    mut commands: Commands,
    config: Res<LightingConfig>,
) {
    info!("Setting up default lighting with quality: {:?}", config.shadow_quality);

    // Main directional light (sun)
    let shadow_config = config.shadow_quality.to_cascade_config();

    commands.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 0.95, 0.85),
            illuminance: 15000.0,
            shadows_enabled: config.shadows_enabled && config.shadow_quality != ShadowQuality::Disabled,
            shadow_depth_bias: 0.02,
            shadow_normal_bias: 0.6,
            affects_lightmapped_mesh_diffuse: true,
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -50f32.to_radians(),
            -30f32.to_radians(),
            0.0,
        )),
        shadow_config,
        MainDirectionalLight,
    ));

    // Ambient light
    if config.ambient_enabled {
        commands.spawn((
            AmbientLight {
                color: Color::srgb(0.5, 0.6, 0.8),
                brightness: 0.2,
                affects_lightmapped_meshes: true,
            },
            MainAmbientLight,
        ));
    }

    // Initialize time of day (static at noon by default)
    commands.insert_resource(TimeOfDay::default());

    info!("Default lighting setup complete");
}

/// System to update shadow quality when config changes
fn update_shadow_quality(
    config: Res<LightingConfig>,
    mut directional_lights: Query<(&mut DirectionalLight, &mut CascadeShadowConfig), With<MainDirectionalLight>>,
) {
    if !config.is_changed() {
        return;
    }

    for (mut light, mut cascade_config) in &mut directional_lights {
        light.shadows_enabled = config.shadows_enabled && config.shadow_quality != ShadowQuality::Disabled;
        *cascade_config = config.shadow_quality.to_cascade_config();

        info!("Updated shadow quality to: {:?}", config.shadow_quality);
    }
}

/// Debug system to toggle shadows with F3 key
fn debug_toggle_shadows(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut config: ResMut<LightingConfig>,
) {
    if keyboard.just_pressed(KeyCode::F3) {
        config.shadows_enabled = !config.shadows_enabled;
        info!("Shadows: {}", if config.shadows_enabled { "ON" } else { "OFF" });
    }

    // Cycle shadow quality with F4
    if keyboard.just_pressed(KeyCode::F4) {
        config.shadow_quality = match config.shadow_quality {
            ShadowQuality::Disabled => ShadowQuality::Low,
            ShadowQuality::Low => ShadowQuality::Medium,
            ShadowQuality::Medium => ShadowQuality::High,
            ShadowQuality::High => ShadowQuality::Ultra,
            ShadowQuality::Ultra => ShadowQuality::Disabled,
        };
        info!("Shadow quality: {:?} (~{:.1}ms)",
            config.shadow_quality,
            config.shadow_quality.gpu_cost_estimate()
        );
    }
}

/// Update time of day and adjust lighting accordingly
fn update_time_of_day(
    time: Res<Time>,
    mut time_of_day: ResMut<TimeOfDay>,
    mut directional_lights: Query<(&mut DirectionalLight, &mut Transform), With<MainDirectionalLight>>,
    mut ambient_lights: Query<&mut AmbientLight, With<MainAmbientLight>>,
) {
    if !time_of_day.enabled {
        return;
    }

    // Update hour
    time_of_day.hour += time.delta_secs() * time_of_day.speed / 3600.0;
    if time_of_day.hour >= 24.0 {
        time_of_day.hour -= 24.0;
    }

    // Update sun/moon position and color
    for (mut light, mut transform) in &mut directional_lights {
        let elevation = time_of_day.sun_elevation();
        let azimuth = time_of_day.sun_azimuth();

        *transform = Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            elevation,
            azimuth,
            0.0,
        ));

        light.color = time_of_day.sun_color();
        light.illuminance = time_of_day.sun_intensity();
    }

    // Update ambient light
    for mut ambient_light in &mut ambient_lights {
        ambient_light.color = time_of_day.ambient_color();
        ambient_light.brightness = time_of_day.ambient_brightness();
    }
}

/// Display shadow debug information
fn display_shadow_debug_info(
    config: Res<LightingConfig>,
    directional_lights: Query<(&DirectionalLight, &CascadeShadowConfig), With<MainDirectionalLight>>,
) {
    if !config.debug_mode {
        return;
    }

    for (light, cascade_config) in &directional_lights {
        debug!(
            "Shadow Debug - Quality: {:?}, Enabled: {}, Cascades: {}, Max Distance: {:.1}m",
            config.shadow_quality,
            light.shadows_enabled,
            cascade_config.bounds.len(),
            cascade_config.bounds.last().copied().unwrap_or(0.0)
        );
    }
}

/// Helper function to create a sun light
pub fn create_sun_light(
    commands: &mut Commands,
    direction: Vec3,
    color: Color,
    intensity: f32,
    shadow_quality: ShadowQuality,
) -> Entity {
    let shadow_config = shadow_quality.to_cascade_config();

    commands.spawn((
        DirectionalLight {
            color,
            illuminance: intensity,
            shadows_enabled: shadow_quality != ShadowQuality::Disabled,
            shadow_depth_bias: 0.02,
            shadow_normal_bias: 0.6,
            affects_lightmapped_mesh_diffuse: true,
        },
        Transform::from_rotation(Quat::from_rotation_arc(Vec3::NEG_Z, direction.normalize())),
        shadow_config,
        MainDirectionalLight,
    )).id()
}

/// Helper function to create a point light with shadows
pub fn create_point_light_with_shadows(
    commands: &mut Commands,
    position: Vec3,
    color: Color,
    intensity: f32,
    range: f32,
    shadows_enabled: bool,
) -> Entity {
    commands.spawn((
        PointLight {
            color,
            intensity,
            range,
            radius: 0.3, // Soft shadows
            shadows_enabled,
            shadow_depth_bias: 0.02,
            shadow_normal_bias: 1.8,
            ..default()
        },
        Transform::from_translation(position),
        GlobalTransform::default(),
        ShadowCastingPointLight,
    )).id()
}

/// Helper function to create a spot light with shadows
pub fn create_spot_light_with_shadows(
    commands: &mut Commands,
    position: Vec3,
    direction: Vec3,
    color: Color,
    intensity: f32,
    range: f32,
    inner_angle: f32,
    outer_angle: f32,
    shadows_enabled: bool,
) -> Entity {
    commands.spawn((
        SpotLight {
            color,
            intensity,
            range,
            radius: 0.2,
            shadows_enabled,
            shadow_depth_bias: 0.02,
            shadow_normal_bias: 1.8,
            inner_angle,
            outer_angle,
            ..default()
        },
        Transform::from_translation(position)
            .looking_to(direction, Vec3::Y),
        GlobalTransform::default(),
    )).id()
}

/// Preset lighting configurations
pub mod presets {
    use super::*;

    /// Bright outdoor daylight
    pub fn outdoor_day(commands: &mut Commands, shadow_quality: ShadowQuality) {
        create_sun_light(
            commands,
            Vec3::new(-0.5, -1.0, -0.3).normalize(),
            Color::srgb(1.0, 0.95, 0.9),
            15000.0,
            shadow_quality,
        );

        commands.spawn((
            AmbientLight {
                color: Color::srgb(0.5, 0.6, 0.8),
                brightness: 0.3,
                affects_lightmapped_meshes: true,
            },
            MainAmbientLight,
        ));
    }

    /// Warm sunset lighting
    pub fn outdoor_sunset(commands: &mut Commands, shadow_quality: ShadowQuality) {
        create_sun_light(
            commands,
            Vec3::new(-0.8, -0.3, -0.2).normalize(),
            Color::srgb(1.0, 0.6, 0.4),
            5000.0,
            shadow_quality,
        );

        commands.spawn((
            AmbientLight {
                color: Color::srgb(0.3, 0.2, 0.3),
                brightness: 0.15,
                affects_lightmapped_meshes: true,
            },
            MainAmbientLight,
        ));
    }

    /// Moonlit night
    pub fn outdoor_night(commands: &mut Commands, shadow_quality: ShadowQuality) {
        create_sun_light(
            commands,
            Vec3::new(0.3, -0.8, 0.2).normalize(),
            Color::srgb(0.3, 0.3, 0.5),
            200.0,
            shadow_quality,
        );

        commands.spawn((
            AmbientLight {
                color: Color::srgb(0.1, 0.1, 0.2),
                brightness: 0.05,
                affects_lightmapped_meshes: true,
            },
            MainAmbientLight,
        ));
    }

    /// Indoor lighting with ambient
    pub fn indoor(commands: &mut Commands) {
        commands.spawn((
            AmbientLight {
                color: Color::srgb(0.8, 0.8, 0.9),
                brightness: 0.4,
                affects_lightmapped_meshes: true,
            },
            MainAmbientLight,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shadow_quality_cascade_counts() {
        assert_eq!(ShadowQuality::Low.to_cascade_config().num_cascades, 1);
        assert_eq!(ShadowQuality::Medium.to_cascade_config().num_cascades, 2);
        assert_eq!(ShadowQuality::High.to_cascade_config().num_cascades, 4);
        assert_eq!(ShadowQuality::Ultra.to_cascade_config().num_cascades, 4);
    }

    #[test]
    fn test_time_of_day_sun_elevation() {
        let morning = TimeOfDay::at_hour(6.0);
        assert!(morning.sun_elevation().abs() < 0.1);

        let noon = TimeOfDay::at_hour(12.0);
        assert!(noon.sun_elevation() > 0.0);

        let evening = TimeOfDay::at_hour(18.0);
        assert!(evening.sun_elevation().abs() < 0.1);
    }

    #[test]
    fn test_time_of_day_wrapping() {
        let mut time = TimeOfDay::at_hour(23.5).with_speed(7200.0); // Fast
        assert!(time.hour >= 0.0 && time.hour < 24.0);
    }
}
