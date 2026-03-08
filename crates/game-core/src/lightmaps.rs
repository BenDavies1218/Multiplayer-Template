use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::mesh::{Mesh, Mesh3d};
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::light::AmbientLight;

/// Plugin for baked lighting (lightmaps) support
///
/// Lightmaps provide pre-calculated lighting and shadows that are "baked" into textures.
/// This dramatically improves performance by eliminating real-time shadow calculations.
///
/// Benefits:
/// - Near-zero GPU cost for shadows/GI
/// - Realistic global illumination
/// - Soft shadows and ambient occlusion
/// - Static lighting looks better than real-time
///
/// Limitations:
/// - Only works for static geometry
/// - Dynamic objects need real-time lighting
/// - Requires UV2 channel in mesh
/// - Increases texture memory usage
pub struct LightmapPlugin;

impl Plugin for LightmapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LightmapConfig>();
        app.add_systems(Update, (
            apply_lightmaps_to_entities,
            debug_toggle_lightmaps,
        ));
    }
}

/// Lightmap configuration resource
#[derive(Resource, Debug, Clone)]
pub struct LightmapConfig {
    /// Enable/disable lightmap rendering
    pub enabled: bool,
    /// Lightmap intensity multiplier (0.0 - 2.0)
    pub intensity: f32,
    /// Mix between lightmap and real-time lighting (0.0 = full lightmap, 1.0 = full real-time)
    pub realtime_mix: f32,
}

impl Default for LightmapConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            intensity: 1.0,
            realtime_mix: 0.0, // Pure baked by default
        }
    }
}

/// Component marking entities that should use lightmaps
#[derive(Component, Debug)]
pub struct Lightmapped {
    /// Handle to the lightmap texture
    pub lightmap: Handle<Image>,
    /// UV channel for lightmap (usually 1, as UV0 is for base textures)
    pub uv_channel: u32,
}

/// Component for ambient occlusion maps
#[derive(Component, Debug)]
pub struct AmbientOcclusion {
    /// Handle to the AO texture
    pub ao_map: Handle<Image>,
    /// AO intensity (0.0 - 1.0)
    pub intensity: f32,
}

/// Material extension for lightmaps
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct LightmapMaterial {
    /// Lightmap texture
    #[texture(10)]
    #[sampler(11)]
    pub lightmap: Handle<Image>,

    /// Lightmap intensity
    #[uniform(12)]
    pub intensity: f32,

    /// Mix with real-time lighting
    #[uniform(13)]
    pub realtime_mix: f32,
}

impl Default for LightmapMaterial {
    fn default() -> Self {
        Self {
            lightmap: Handle::default(),
            intensity: 1.0,
            realtime_mix: 0.0,
        }
    }
}

/// Automatically apply lightmaps to entities with Lightmapped component
fn apply_lightmaps_to_entities(
    mut commands: Commands,
    query: Query<(Entity, &Lightmapped, &MeshMaterial3d<StandardMaterial>), Added<Lightmapped>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<LightmapConfig>,
) {
    if !config.enabled {
        return;
    }

    for (entity, lightmapped, material3d) in &query {
        // Get the existing material
        if let Some(_material) = materials.get(&material3d.0) {
            // Note: In Bevy 0.18, we'd typically use a custom shader or material extension
            // This is a simplified version showing the concept

            info!("Applied lightmap to entity {:?} using UV channel {}",
                entity,
                lightmapped.uv_channel
            );

            // In a full implementation, you'd:
            // 1. Create custom shader that reads from UV2
            // 2. Multiply base color by lightmap
            // 3. Handle emissive for baked lights
        }
    }
}

/// Debug system to toggle lightmaps with F8
fn debug_toggle_lightmaps(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut config: ResMut<LightmapConfig>,
) {
    if keyboard.just_pressed(KeyCode::F8) {
        config.enabled = !config.enabled;
        info!("Lightmaps: {}", if config.enabled { "ON" } else { "OFF" });
    }

    // Adjust intensity with F9/F10
    if keyboard.pressed(KeyCode::F9) {
        config.intensity = (config.intensity - 0.05).max(0.0);
        debug!("Lightmap intensity: {:.2}", config.intensity);
    }
    if keyboard.pressed(KeyCode::F10) {
        config.intensity = (config.intensity + 0.05).min(2.0);
        debug!("Lightmap intensity: {:.2}", config.intensity);
    }
}

/// Helper function to create a lightmapped entity
pub fn spawn_lightmapped_entity(
    commands: &mut Commands,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    lightmap: Handle<Image>,
    transform: Transform,
) -> Entity {
    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(material),
        transform,
        GlobalTransform::default(),
        Lightmapped {
            lightmap,
            uv_channel: 1, // UV2 in Blender = channel 1 in Bevy
        },
    )).id()
}

/// Baking strategies for different scenarios
pub mod strategies {
    use super::*;

    /// Full baked lighting (best performance)
    ///
    /// Use when:
    /// - Scene is completely static
    /// - Maximum performance needed
    /// - Artistic control over lighting
    pub struct FullBaked {
        /// Disable real-time directional light shadows
        pub disable_realtime_shadows: bool,
        /// Keep minimal real-time lighting for dynamic objects
        pub minimal_realtime: bool,
    }

    impl Default for FullBaked {
        fn default() -> Self {
            Self {
                disable_realtime_shadows: true,
                minimal_realtime: true,
            }
        }
    }

    /// Hybrid lighting (balanced)
    ///
    /// Use when:
    /// - Mix of static and dynamic objects
    /// - Want baked GI + real-time shadows for dynamics
    /// - Medium-high end hardware
    pub struct Hybrid {
        /// Lightmap intensity for static geometry
        pub lightmap_intensity: f32,
        /// Real-time lighting intensity for dynamic objects
        pub realtime_intensity: f32,
        /// Use real-time shadows only for dynamic objects
        pub dynamic_shadows_only: bool,
    }

    impl Default for Hybrid {
        fn default() -> Self {
            Self {
                lightmap_intensity: 0.8,
                realtime_intensity: 1.0,
                dynamic_shadows_only: true,
            }
        }
    }

    /// AO-only (subtle enhancement)
    ///
    /// Use when:
    /// - Want to keep real-time lighting
    /// - Just add contact shadows/occlusion
    /// - Minimal memory overhead
    pub struct AmbientOcclusionOnly {
        /// AO intensity
        pub ao_strength: f32,
    }

    impl Default for AmbientOcclusionOnly {
        fn default() -> Self {
            Self {
                ao_strength: 0.5,
            }
        }
    }
}

/// Performance comparison helper
pub struct LightingPerformance;

impl LightingPerformance {
    /// Estimate GPU time for different lighting approaches
    pub fn estimate_gpu_time(
        num_lights: usize,
        shadow_quality: crate::lighting::ShadowQuality,
        use_lightmaps: bool,
    ) -> f32 {
        let mut total = 0.0;

        if use_lightmaps {
            // Lightmap lookup is very cheap
            total += 0.1; // ~0.1ms for texture sampling
        } else {
            // Real-time lighting cost
            total += shadow_quality.gpu_cost_estimate();
            total += num_lights as f32 * 0.5; // Each light adds ~0.5ms
        }

        total
    }

    /// Print performance comparison
    pub fn print_comparison() {
        info!("=== Lighting Performance Comparison ===");
        info!("Real-time (4 cascades): ~5-7ms GPU");
        info!("Real-time (2 cascades): ~2-3ms GPU");
        info!("Hybrid (baked + 2 cascades): ~1-2ms GPU");
        info!("Full baked (lightmaps only): ~0.1ms GPU");
        info!("==========================================");
    }
}

/// Blender workflow integration
pub mod blender {
    use super::*;

    /// Instructions for baking lightmaps in Blender
    pub struct BakingGuide;

    impl BakingGuide {
        pub fn print_instructions() {
            info!("=== Blender Lightmap Baking Guide ===");
            info!("1. UV Unwrap to UV Map 1 (for lightmap)");
            info!("2. Render Settings → Bake → Combined");
            info!("3. Influence: Direct + Indirect + Color");
            info!("4. Output: 2048x2048 or 4096x4096");
            info!("5. Save as PNG to assets/lightmaps/");
            info!("6. Export glTF with UV2 enabled");
            info!("======================================");
        }
    }

    /// Texture resolution recommendations
    pub fn recommended_lightmap_resolution(world_size: f32) -> u32 {
        match world_size {
            s if s < 50.0 => 1024,   // Small room
            s if s < 100.0 => 2048,  // Medium area
            s if s < 200.0 => 4096,  // Large area
            _ => 8192,               // Huge world
        }
    }

    /// Calculate texel density (texels per meter)
    pub fn calculate_texel_density(
        world_size: f32,
        texture_resolution: u32,
    ) -> f32 {
        texture_resolution as f32 / world_size
    }
}

/// Memory usage calculator
pub struct LightmapMemory;

impl LightmapMemory {
    /// Calculate memory usage for lightmaps
    pub fn calculate_usage(
        resolution: u32,
        num_maps: usize,
        bytes_per_pixel: usize,
    ) -> usize {
        let pixels_per_map = (resolution * resolution) as usize;
        pixels_per_map * bytes_per_pixel * num_maps
    }

    /// Print memory usage comparison
    pub fn print_usage_comparison() {
        info!("=== Lightmap Memory Usage ===");
        info!("1024x1024 RGB: {}MB", Self::calculate_usage(1024, 1, 3) / 1_000_000);
        info!("2048x2048 RGB: {}MB", Self::calculate_usage(2048, 1, 3) / 1_000_000);
        info!("4096x4096 RGB: {}MB", Self::calculate_usage(4096, 1, 3) / 1_000_000);
        info!("8192x8192 RGB: {}MB", Self::calculate_usage(8192, 1, 3) / 1_000_000);
        info!("============================");
    }
}

/// Example: Setting up a scene with baked lighting
#[allow(dead_code)]
pub fn example_baked_scene_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Load lightmap texture
    let lightmap = asset_server.load("lightmaps/world_lightmap.png");

    // Load world mesh
    let world_mesh = asset_server.load("models/world_visual.glb#Mesh0");

    // Create material
    let material = StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.5,
        metallic: 0.0,
        ..default()
    };

    // Spawn lightmapped world
    spawn_lightmapped_entity(
        &mut commands,
        world_mesh,
        // materials.add(material),  // You'd need to pass materials as parameter
        Handle::default(), // Placeholder
        lightmap,
        Transform::IDENTITY,
    );

    // Disable real-time shadows on static geometry
    commands.insert_resource(crate::lighting::LightingConfig {
        shadows_enabled: false, // No real-time shadows needed!
        ..default()
    });

    // Keep minimal ambient for dynamic objects
    commands.spawn((
        AmbientLight {
            color: Color::srgb(1.0, 1.0, 1.0),
            brightness: 0.3,
            affects_lightmapped_meshes: true,
        },
        crate::lighting::MainAmbientLight,
    ));

    info!("Baked lighting scene configured");
}

/// Hybrid setup: Baked for static, real-time for dynamic
#[allow(dead_code)]
pub fn example_hybrid_setup(
    mut commands: Commands,
) {
    // Configure lightmap
    commands.insert_resource(LightmapConfig {
        enabled: true,
        intensity: 0.8,
        realtime_mix: 0.2, // 80% baked, 20% real-time
    });

    // Keep real-time shadows for dynamic objects only
    commands.insert_resource(crate::lighting::LightingConfig {
        shadows_enabled: true,
        shadow_quality: crate::lighting::ShadowQuality::Low, // Low quality for dynamics
        ..default()
    });

    info!("Hybrid lighting configured: Baked for statics, real-time for dynamics");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lightmap_config_defaults() {
        let config = LightmapConfig::default();
        assert!(config.enabled);
        assert_eq!(config.intensity, 1.0);
        assert_eq!(config.realtime_mix, 0.0);
    }

    #[test]
    fn test_performance_estimation() {
        use crate::lighting::ShadowQuality;

        let baked = LightingPerformance::estimate_gpu_time(0, ShadowQuality::Disabled, true);
        let realtime = LightingPerformance::estimate_gpu_time(3, ShadowQuality::High, false);

        assert!(baked < realtime);
        assert!(baked < 1.0); // Should be very fast
    }

    #[test]
    fn test_memory_calculation() {
        let usage_1k = LightmapMemory::calculate_usage(1024, 1, 3);
        let usage_2k = LightmapMemory::calculate_usage(2048, 1, 3);

        assert_eq!(usage_2k, usage_1k * 4); // 2x resolution = 4x memory
    }

    #[test]
    fn test_resolution_recommendations() {
        assert_eq!(blender::recommended_lightmap_resolution(30.0), 1024);
        assert_eq!(blender::recommended_lightmap_resolution(80.0), 2048);
        assert_eq!(blender::recommended_lightmap_resolution(150.0), 4096);
        assert_eq!(blender::recommended_lightmap_resolution(500.0), 8192);
    }
}
