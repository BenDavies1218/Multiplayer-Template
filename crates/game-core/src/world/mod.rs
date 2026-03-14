use avian3d::prelude::*;
use bevy::mesh::Mesh;
use bevy::prelude::*;

// Module declarations
pub mod collision_debug;
mod loader;
mod processor;
#[cfg(test)]
mod tests;
mod utils;

// Re-export public items from submodules
pub use collision_debug::{CollisionDebugMesh, CollisionDebugSettings, apply_debug_config};
pub use loader::load_world_assets;
pub use processor::{create_compound_collider, create_convex_hull_collider};
pub use utils::{extract_mesh_indices, extract_mesh_vertices, parse_extras};

/// Configuration for WorldPlugin
///
/// Controls whether visual meshes are loaded. Collision is now handled by ZonePlugin.
#[derive(Resource, Debug, Clone)]
pub struct WorldPluginConfig {
    /// Whether to load visual meshes (high-poly glTF/GLB)
    pub load_visual: bool,
}

impl Default for WorldPluginConfig {
    fn default() -> Self {
        Self { load_visual: true }
    }
}

impl WorldPluginConfig {
    /// Server configuration: no visuals
    pub fn server() -> Self {
        Self { load_visual: false }
    }

    /// Client configuration: load visuals
    pub fn client() -> Self {
        Self { load_visual: true }
    }

    /// World viewer configuration: load visuals
    pub fn viewer() -> Self {
        Self { load_visual: true }
    }
}

/// Plugin for loading world visual assets from Blender exports.
///
/// Handles loading visual meshes (high-poly glTF/GLB files).
/// Collision and zone processing is handled by `ZonePlugin`.
pub struct WorldPlugin {
    pub config: WorldPluginConfig,
}

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.config.clone());
        app.init_resource::<WorldAssets>();

        // Add loader system
        app.add_systems(Startup, load_world_assets);
    }
}

/// Marker component for world visual entities
#[derive(Component, Debug)]
pub struct WorldVisual;

/// Resource to store handles to world assets
#[derive(Resource, Debug, Default)]
pub struct WorldAssets {
    pub visual: Option<Handle<Scene>>,
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
