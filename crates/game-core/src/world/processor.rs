use avian3d::prelude::*;
use bevy::gltf::{Gltf, GltfMesh, GltfNode};
use bevy::mesh::Mesh;
use bevy::prelude::*;

use super::collision_debug::{CollisionDebugMesh, CollisionDebugSettings};
use super::utils::extract_mesh_vertices;
use super::{WorldCollisionBundle, WorldCollisionLoader, WorldPluginConfig};

/// Process collision meshes from loaded collision file
///
/// This system watches for WorldCollisionLoader entities, waits for their assets
/// to load, then extracts meshes and creates Avian3d colliders.
///
/// IMPORTANT: This processes the glTF scene hierarchy to preserve node transforms
/// (position, rotation, scale) from Blender exports.
#[allow(clippy::too_many_arguments)]
pub fn process_collision_meshes(
    mut commands: Commands,
    collision_query: Query<(Entity, &WorldCollisionLoader)>,
    gltf_assets: Res<Assets<Gltf>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
    gltf_nodes: Res<Assets<GltfNode>>,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    mut materials: Option<ResMut<Assets<StandardMaterial>>>,
    config: Res<WorldPluginConfig>,
    debug_settings: Option<Res<CollisionDebugSettings>>,
) {
    for (entity, loader) in collision_query.iter() {
        // Check if asset is loaded
        let Some(gltf) = gltf_assets.get(&loader.handle) else {
            continue; // Still loading
        };

        // Log all node names first for debugging
        for (node_name, _) in &gltf.named_nodes {
            info!("  Collision node available: '{}'", node_name);
        }

        // Process all named nodes in the glTF file
        // This preserves the transform hierarchy from Blender
        for (node_name, node_handle) in &gltf.named_nodes {
            let Some(gltf_node) = gltf_nodes.get(node_handle) else {
                warn!("GltfNode not found for {}", node_name);
                continue;
            };

            // Get the node's transform
            let node_transform = gltf_node.transform;

            // Check if the node has a mesh reference (some nodes may be empty or just for grouping)
            let Some(gltf_mesh_handle) = &gltf_node.mesh else {
                continue;
            };

            let Some(gltf_mesh) = gltf_meshes.get(gltf_mesh_handle) else {
                warn!("GltfMesh not found for node {}", node_name);
                continue;
            };

            // Process all primitives in the mesh
            for primitive in &gltf_mesh.primitives {
                // Check the node has a primitive with a mesh reference
                // Get mesh from either the mutable or immutable resource
                let mesh_handle = &primitive.mesh;
                let mesh = if let Some(ref mesh_res) = meshes {
                    mesh_res.get(mesh_handle)
                } else {
                    None
                };

                let Some(mesh) = mesh else {
                    warn!(
                        "Mesh primitive not found for node {} or Mesh assets not available",
                        node_name
                    );
                    continue;
                };

                // Clone mesh data early for potential debug visualization
                // This allows us to release the immutable borrow before taking a mutable one
                let mesh_clone_for_debug = mesh.clone();

                // Create trimesh collider from the mesh
                let collision_transform = node_transform;

                if let Some(bundle) = WorldCollisionBundle::from_mesh(mesh, collision_transform) {
                    // Spawn collision entity
                    let mut entity_commands =
                        commands.spawn((bundle, Name::new(format!("Collision: {}", node_name))));

                    // Only create debug visualization if enabled and resources are available
                    if config.enable_debug
                        && let (Some(settings), Some(mesh_res), Some(mat_res)) =
                            (&debug_settings, &mut meshes, &mut materials)
                    {
                        // Use the pre-cloned mesh for visualization
                        let debug_mesh_handle = mesh_res.add(mesh_clone_for_debug);

                        // Create semi-transparent material
                        let debug_material = mat_res.add(StandardMaterial {
                            base_color: settings.color,
                            alpha_mode: AlphaMode::Blend,
                            double_sided: true,
                            cull_mode: None,
                            ..default()
                        });

                        // Add debug mesh as child
                        entity_commands.with_children(|parent| {
                            parent.spawn((
                                Mesh3d(debug_mesh_handle),
                                MeshMaterial3d(debug_material),
                                Transform::default(), // Use parent's transform
                                GlobalTransform::default(),
                                Visibility::Hidden, // Start hidden
                                CollisionDebugMesh,
                                Name::new(format!("DebugMesh: {}", node_name)),
                            ));
                        });
                    }
                } else {
                    error!("Failed to create collider from node {}", node_name);
                }
            }
        }

        // Remove the loader entity after processing
        commands.entity(entity).despawn();
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

/// Create a compound collider from multiple simple shapes
///
/// This is useful when you want to manually define collision using basic shapes
/// instead of mesh-based collision. Better performance for simple geometry.
///
/// Example:
/// ```rust
/// use avian3d::prelude::Collider;
/// use bevy::prelude::{Quat, Vec3};
/// use game_core::world::create_compound_collider;
/// let collider = create_compound_collider(vec![
///     (Vec3::ZERO, Quat::IDENTITY, Collider::cuboid(10.0, 1.0, 10.0)), // floor
///     (Vec3::new(5.0, 2.0, 0.0), Quat::IDENTITY, Collider::cuboid(1.0, 2.0, 10.0)), // wall
/// ]);
/// ```
pub fn create_compound_collider(shapes: Vec<(Vec3, Quat, Collider)>) -> Collider {
    Collider::compound(shapes)
}
