use avian3d::prelude::*;
use bevy::mesh::Mesh;
use bevy::prelude::*;

use super::utils::extract_mesh_vertices;

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
