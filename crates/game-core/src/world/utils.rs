use bevy::prelude::*;
use bevy::gltf::GltfExtras;
use bevy::mesh::{Mesh, VertexAttributeValues, Indices};
use std::collections::HashMap;

/// Extract vertices from a Bevy mesh
pub fn extract_mesh_vertices(mesh: &Mesh) -> Option<Vec<Vec3>> {
    let positions = mesh.attribute(Mesh::ATTRIBUTE_POSITION)?;

    match positions {
        VertexAttributeValues::Float32x3(vertices) => {
            Some(vertices.iter().map(|v| Vec3::from(*v)).collect())
        }
        _ => None,
    }
}

/// Extract indices from a Bevy mesh
pub fn extract_mesh_indices(mesh: &Mesh) -> Option<Vec<[u32; 3]>> {
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

/// Parse glTF extras JSON into a HashMap
pub fn parse_extras(extras: &Option<GltfExtras>) -> HashMap<String, serde_json::Value> {
    extras
        .as_ref()
        .and_then(|e| serde_json::from_str(&e.value).ok())
        .unwrap_or_default()
}
