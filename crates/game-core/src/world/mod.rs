use avian3d::prelude::*;
use bevy::gltf::Gltf;
use bevy::mesh::Mesh;
use bevy::prelude::*;

// Module declarations
mod collision_debug;
mod loader;
mod processor;
#[cfg(test)]
mod tests;
mod utils;

// Re-export public items from submodules
pub use collision_debug::{CollisionDebugMesh, CollisionDebugSettings, apply_debug_config};
pub use loader::load_world_assets;
pub use processor::{
    create_compound_collider, create_convex_hull_collider, process_collision_meshes,
};
pub use utils::{extract_mesh_indices, extract_mesh_vertices, parse_extras};

/// Configuration for WorldPlugin
///
/// Controls what assets are loaded and what features are enabled.
#[derive(Resource, Debug, Clone)]
pub struct WorldPluginConfig {
    /// Whether to load visual meshes (high-poly glTF/GLB)
    pub load_visual: bool,
    /// Whether to load collision meshes (low-poly glTF/GLB)
    pub load_collision: bool,
    /// Whether to enable collision debug visualization
    pub enable_debug: bool,
}

impl Default for WorldPluginConfig {
    fn default() -> Self {
        Self {
            load_visual: true,
            load_collision: true,
            enable_debug: true,
        }
    }
}

impl WorldPluginConfig {
    /// Server configuration: collision only, no visuals, no debug
    pub fn server() -> Self {
        Self {
            load_visual: false,
            load_collision: true,
            enable_debug: false,
        }
    }

    /// Client configuration: full features no debug
    pub fn client() -> Self {
        Self {
            load_visual: true,
            load_collision: true,
            enable_debug: false,
        }
    }

    /// World viewer configuration: full features with debug
    pub fn viewer() -> Self {
        Self {
            load_visual: true,
            load_collision: true,
            enable_debug: true,
        }
    }
}

/// Plugin for loading and managing world assets from Blender exports.
///
/// This plugin handles:
/// - Loading visual meshes (high-poly glTF/GLB files)
/// - Loading collision meshes (low-poly glTF/GLB files)
/// - Converting Blender meshes to Avian3d colliders
/// - Automatic collision generation from naming conventions
///
/// Use `WorldPluginConfig` to customize what gets loaded for different environments.
pub struct WorldPlugin {
    pub config: WorldPluginConfig,
}

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        // Insert configuration as resource
        app.insert_resource(self.config.clone());
        app.init_resource::<WorldAssets>();

        // Only initialize debug settings if debug is enabled
        if self.config.enable_debug {
            app.init_resource::<CollisionDebugSettings>();
        }

        // Add loader system
        app.add_systems(Startup, load_world_assets);

        // Add collision processing system
        app.add_systems(Update, process_collision_meshes);

        // Only add debug systems if debug is enabled
        if self.config.enable_debug {
            app.add_systems(Startup, apply_debug_config);
            app.add_systems(
                Update,
                (
                    collision_debug::toggle_collision_debug,
                    collision_debug::update_collision_debug_visibility,
                ),
            );
        }
    }
}

/// Marker component for world visual entities
#[derive(Component, Debug)]
pub struct WorldVisual;

/// Component for collision mesh loaders (temporary, removed after processing)
#[derive(Component, Debug)]
pub struct WorldCollisionLoader {
    pub handle: Handle<Gltf>,
}

/// Resource to store handles to world assets
#[derive(Resource, Debug)]
pub struct WorldAssets {
    pub visual: Option<Handle<Scene>>, // Handle to the visual scene (high-poly mesh)
    pub collision: Option<Handle<Gltf>>, // Handle to the collision mesh (low-poly mesh)
}

impl Default for WorldAssets {
    fn default() -> Self {
        Self {
            visual: None,
            collision: None,
        }
    }
}

/// Bundle for spawning world collision entities
#[derive(Bundle)]
pub struct WorldCollisionBundle {
    pub collider: Collider,
    pub rigid_body: RigidBody,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl WorldCollisionBundle {
    /// Create a new collision bundle from a mesh
    /// Uses trimesh collider for complex geometry
    pub fn from_mesh(mesh: &Mesh, transform: Transform) -> Option<Self> {
        // Extract vertices and indices from mesh
        let vertices = extract_mesh_vertices(mesh)?;
        let indices = extract_mesh_indices(mesh)?;

        Some(Self {
            collider: Collider::trimesh(vertices, indices),
            rigid_body: RigidBody::Static,
            transform,
            global_transform: GlobalTransform::default(),
        })
    }

    /// Create a collision bundle with a specific collider type
    pub fn new(collider: Collider, transform: Transform) -> Self {
        Self {
            collider,
            rigid_body: RigidBody::Static,
            transform,
            global_transform: GlobalTransform::default(),
        }
    }
}
