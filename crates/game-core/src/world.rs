use avian3d::prelude::*;
use bevy::prelude::*;
use bevy::gltf::{Gltf, GltfMesh, GltfNode};
use bevy::mesh::{Mesh, Mesh3d, VertexAttributeValues, Indices};
use bevy::scene::{Scene, SceneRoot};

/// Plugin for loading and managing world assets from Blender exports.
///
/// This plugin handles:
/// - Loading visual meshes (high-poly glTF/GLB files)
/// - Loading collision meshes (low-poly glTF/GLB files)
/// - Converting Blender meshes to Avian3d colliders
/// - Automatic collision generation from naming conventions
pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldAssets>();
        app.add_systems(Startup, load_world_assets);
        app.add_systems(Update, (
            process_collision_meshes,
            prevent_visual_colliders,
        ));
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

/// Extract vertices from a Bevy mesh
fn extract_mesh_vertices(mesh: &Mesh) -> Option<Vec<Vec3>> {
    let positions = mesh.attribute(Mesh::ATTRIBUTE_POSITION)?;

    match positions {
        VertexAttributeValues::Float32x3(vertices) => {
            Some(vertices.iter().map(|v| Vec3::from(*v)).collect())
        }
        _ => None,
    }
}

/// Extract indices from a Bevy mesh
fn extract_mesh_indices(mesh: &Mesh) -> Option<Vec<[u32; 3]>> {
    let indices = mesh.indices()?;

    match indices {
        Indices::U32(idx) => {
            Some(idx.chunks_exact(3)
                .map(|chunk| [chunk[0], chunk[1], chunk[2]])
                .collect())
        }
        Indices::U16(idx) => {
            Some(idx.chunks_exact(3)
                .map(|chunk| [chunk[0] as u32, chunk[1] as u32, chunk[2] as u32])
                .collect())
        }
    }
}

/// Load world assets at startup
///
/// To use this system, place your world files in:
/// - `assets/models/example_world_visual.glb` - High-poly visual mesh
/// - `assets/models/example_world_collision.glb` - Low-poly collision mesh
///
/// You can customize the paths by modifying this function or creating your own loader.
fn load_world_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Load world visual scene
    let visual_handle = asset_server.load(format!("{}#Scene0", crate::shared::WORLD_VISUAL_PATH));

    commands.spawn((
        SceneRoot(visual_handle.clone()),
        WorldVisual,
        Transform::default(),
        GlobalTransform::default(),
    ));

    // Load collision mesh for processing
    let collision_handle: Handle<Gltf> = asset_server.load(crate::shared::WORLD_COLLISION_PATH);
    commands.spawn(WorldCollisionLoader {
        handle: collision_handle.clone(),
    });

    // Store handles in resource for later access
    commands.insert_resource(WorldAssets {
        visual: Some(visual_handle),
        collision: Some(collision_handle),
    });

    info!("Loading world assets from {} and {}",
        crate::shared::WORLD_VISUAL_PATH,
        crate::shared::WORLD_COLLISION_PATH
    );
}

/// Prevents physics colliders from being added to visual mesh entities.
///
/// This system removes any Collider components from entities that are children
/// of WorldVisual entities. This ensures that only the low-poly collision mesh
/// is used for physics, not the high-poly visual mesh.
fn prevent_visual_colliders(
    mut commands: Commands,
    visual_query: Query<Entity, With<WorldVisual>>,
    children_query: Query<&Children>,
    collider_query: Query<Entity, (With<Collider>, Without<WorldCollisionLoader>)>,
) {
    for visual_entity in visual_query.iter() {
        // Remove colliders from the visual entity itself
        if collider_query.contains(visual_entity) {
            warn!("Removing collider from WorldVisual entity");
            commands.entity(visual_entity).remove::<Collider>();
        }

        // Remove colliders from all children (recursively)
        remove_colliders_from_children(
            &mut commands,
            visual_entity,
            &children_query,
            &collider_query,
        );
    }
}

/// Recursively removes colliders from all children of an entity
fn remove_colliders_from_children(
    commands: &mut Commands,
    entity: Entity,
    children_query: &Query<&Children>,
    collider_query: &Query<Entity, (With<Collider>, Without<WorldCollisionLoader>)>,
) {
    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            // Remove collider from this child if it has one
            if collider_query.contains(child) {
                info!("Removing collider from visual mesh child entity");
                commands.entity(child).remove::<Collider>();
            }

            // Recursively check this child's children
            remove_colliders_from_children(commands, child, children_query, collider_query);
        }
    }
}

/// Process collision meshes from loaded glTF files
///
/// This system watches for WorldCollisionLoader entities, waits for their assets
/// to load, then extracts meshes and creates Avian3d colliders.
///
/// IMPORTANT: This processes the glTF scene hierarchy to preserve node transforms
/// (position, rotation, scale) from Blender exports.
fn process_collision_meshes(
    mut commands: Commands,
    collision_query: Query<(Entity, &WorldCollisionLoader)>,
    gltf_assets: Res<Assets<Gltf>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
    gltf_nodes: Res<Assets<GltfNode>>,
    meshes: Res<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, loader) in collision_query.iter() {
        // Check if asset is loaded
        let Some(gltf) = gltf_assets.get(&loader.handle) else {
            continue; // Still loading
        };

        info!("Processing collision mesh from glTF");

        // Process all named nodes in the glTF file
        // This preserves the transform hierarchy from Blender
        for (node_name, node_handle) in &gltf.named_nodes {
            // Skip nodes that shouldn't have colliders
            // You can customize this to filter out specific nodes by name
            // For example, skip nodes named "floor" or nodes starting with "visual_"
            if should_skip_collision_node(node_name) {
                info!("Skipping collision for node '{}' (filtered out)", node_name);
                continue;
            }

            let Some(gltf_node) = gltf_nodes.get(node_handle) else {
                warn!("GltfNode not found for {}", node_name);
                continue;
            };

            // Get the node's transform from Blender
            let node_transform = gltf_node.transform;
            info!("Processing node '{}' with transform: {:?}", node_name, node_transform);

            // Get the mesh if this node has one
            let Some(gltf_mesh_handle) = &gltf_node.mesh else {
                info!("Node '{}' has no mesh, skipping", node_name);
                continue;
            };

            let Some(gltf_mesh) = gltf_meshes.get(gltf_mesh_handle) else {
                warn!("GltfMesh not found for node {}", node_name);
                continue;
            };

            // Process all primitives in the mesh
            for primitive in &gltf_mesh.primitives {
                let Some(mesh) = meshes.get(&primitive.mesh) else {
                    warn!("Mesh primitive not found for node {}", node_name);
                    continue;
                };

                // Create trimesh collider from the mesh
                // Use the node's transform directly from Blender (already in correct Y-up coordinate system)
                let collision_transform = node_transform;

                info!("Final collision transform for '{}': {:?}", node_name, collision_transform);

                if let Some(bundle) = WorldCollisionBundle::from_mesh(mesh, collision_transform) {
                    info!("Created trimesh collider for node: {}", node_name);

                    // Spawn with both physics collider AND visual mesh
                    // Use the material from the GLB file if available, but modify it for double-sided rendering
                    let collision_material = if let Some(material_handle) = &primitive.material {
                        // Clone the existing material and make it double-sided
                        let cloned_mat = materials.get(material_handle).cloned();
                        if let Some(mut existing_mat) = cloned_mat {
                            info!("Using existing material for '{}' (color: {:?})", node_name, existing_mat.base_color);
                            existing_mat.double_sided = true; // Render from both sides
                            existing_mat.cull_mode = None;    // Don't cull any faces
                            materials.add(existing_mat)
                        } else {
                            info!("Material handle exists but couldn't get material data for {}", node_name);
                            material_handle.clone()
                        }
                    } else {
                        info!("No material found for {}, using default red material", node_name);
                        // No material in GLB, create a simple default double-sided one
                        materials.add(StandardMaterial {
                            base_color: Color::srgba(1.0, 0.0, 0.0, 0.5), // Semi-transparent red
                            alpha_mode: AlphaMode::Blend,
                            double_sided: true,
                            cull_mode: None,
                            ..default()
                        })
                    };

                    commands.spawn((
                        bundle,
                        Mesh3d(primitive.mesh.clone()),
                        MeshMaterial3d(collision_material),
                        Name::new(format!("Collision: {}", node_name)),
                    ));
                } else {
                    error!("Failed to create collider from node {}", node_name);
                }
            }
        }

        // Remove the loader entity after processing
        commands.entity(entity).despawn();
        info!("Collision mesh processing complete");
    }
}

/// Helper function to create a convex hull collider from a mesh
///
/// Use this for dynamic objects or when you need better performance than trimesh.
/// Convex hulls are faster but can only approximate concave shapes.
pub fn create_convex_hull_collider(mesh: &Mesh) -> Option<Collider> {
    let vertices = extract_mesh_vertices(mesh)?;
    Collider::convex_hull(vertices)
}

/// Helper function to load a world visual scene
///
/// Example usage:
/// ```rust
/// fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
///     load_world_visual(&mut commands, &asset_server, "models/castle.glb", Vec3::ZERO);
/// }
/// ```
pub fn load_world_visual(
    commands: &mut Commands,
    asset_server: &AssetServer,
    path: &str,
    position: Vec3,
) {
    let scene_handle = asset_server.load(format!("{}#Scene0", path));

    commands.spawn((
        SceneRoot(scene_handle),
        WorldVisual,
        Transform::from_translation(position),
        GlobalTransform::default(),
    ));

    info!("Loading world visual from: {}", path);
}

/// Helper function to load a world collision mesh
///
/// Example usage:
/// ```rust
/// fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
///     load_world_collision(&mut commands, &asset_server, "models/castle_collision.glb");
/// }
/// ```
pub fn load_world_collision(
    commands: &mut Commands,
    asset_server: &AssetServer,
    path: &str,
) {
    let collision_handle: Handle<Gltf> = asset_server.load(path.to_string());

    commands.spawn(WorldCollisionLoader {
        handle: collision_handle,
    });

    info!("Loading world collision from: {}", path);
}

/// Example system showing how to load a complete world (visual + collision)
///
/// You can copy this into your own code and customize the paths.
#[allow(dead_code)]
fn example_load_complete_world(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Load visual mesh
    load_world_visual(
        &mut commands,
        &asset_server,
        "models/my_world_visual.glb",
        Vec3::ZERO,
    );

    // Load collision mesh
    load_world_collision(
        &mut commands,
        &asset_server,
        "models/my_world_collision.glb",
    );
}

/// Determines whether a node should be skipped when creating collision meshes.
///
/// You can customize this function to filter out specific nodes from your collision GLB file.
/// By default, it skips nodes named "floor" (case-insensitive).
///
/// # Filtering Strategies
///
/// 1. **By exact name:** Skip specific objects
///    ```rust
///    node_name.eq_ignore_ascii_case("floor") || node_name.eq_ignore_ascii_case("light")
///    ```
///
/// 2. **By prefix:** Skip all nodes starting with a prefix
///    ```rust
///    node_name.starts_with("visual_") || node_name.starts_with("ignore_")
///    ```
///
/// 3. **By suffix:** Only include nodes ending with specific suffix
///    ```rust
///    !node_name.ends_with("_collision")
///    ```
///
/// 4. **Allowlist approach:** Only process specific nodes
///    ```rust
///    !matches!(node_name, "ground" | "walls" | "obstacles")
///    ```
fn should_skip_collision_node(node_name: &str) -> bool {
    // Skip nodes named "floor" (case-insensitive)
    // Add more conditions here as needed
    node_name.eq_ignore_ascii_case("floor")

    // Alternative filtering examples (uncomment to use):

    // Skip anything starting with "visual_" or "ignore_"
    // || node_name.starts_with("visual_")
    // || node_name.starts_with("ignore_")

    // Only process nodes ending with "_collision"
    // || !node_name.ends_with("_collision")
}

/// Create a compound collider from multiple simple shapes
///
/// This is useful when you want to manually define collision using basic shapes
/// instead of mesh-based collision. Better performance for simple geometry.
///
/// Example:
/// ```rust
/// let collider = create_compound_collider(vec![
///     (Vec3::ZERO, Quat::IDENTITY, Collider::cuboid(10.0, 1.0, 10.0)), // floor
///     (Vec3::new(5.0, 2.0, 0.0), Quat::IDENTITY, Collider::cuboid(1.0, 2.0, 10.0)), // wall
/// ]);
/// ```
pub fn create_compound_collider(shapes: Vec<(Vec3, Quat, Collider)>) -> Collider {
    Collider::compound(shapes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_assets_default() {
        let assets = WorldAssets::default();
        assert!(assets.visual.is_none());
        assert!(assets.collision.is_none());
    }

    #[test]
    fn test_compound_collider_creation() {
        let _collider = create_compound_collider(vec![
            (Vec3::ZERO, Quat::IDENTITY, Collider::cuboid(1.0, 1.0, 1.0)),
        ]);

        // Just verify it compiles
        assert!(true);
    }
}
