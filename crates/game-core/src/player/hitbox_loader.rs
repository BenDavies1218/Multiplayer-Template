use avian3d::prelude::*;
use bevy::prelude::*;
use bevy::gltf::{Gltf, GltfMesh, GltfNode};
use bevy::mesh::Mesh;
use crate::world::{extract_mesh_vertices, extract_mesh_indices, parse_extras};
use super::components::{PlayerHitboxMarker, HitboxRegion};

/// Temporary component placed on a loader entity.
/// Removed after the GLB is processed into `PlayerHitboxData`.
#[derive(Component, Debug)]
pub struct PlayerHitboxLoader {
    pub handle: Handle<Gltf>,
}

/// A single parsed hitbox region ready to be attached to player entities.
#[derive(Clone, Debug)]
pub struct HitboxRegionData {
    pub name: String,
    pub base_damage: f32,
    pub collider: Collider,
    pub transform: Transform,
}

/// Resource holding the parsed hitbox regions.
/// Inserted once the hitbox GLB has been processed.
/// Used by the server when spawning players to attach hitbox children.
#[derive(Resource, Debug, Clone)]
pub struct PlayerHitboxData {
    pub regions: Vec<HitboxRegionData>,
}

/// System that processes the hitbox GLB once it's loaded.
/// Runs every frame until the loader entity is found and the asset is ready.
/// After processing, inserts `PlayerHitboxData` resource and despawns the loader.
pub fn process_player_hitbox(
    mut commands: Commands,
    loader_query: Query<(Entity, &PlayerHitboxLoader)>,
    gltf_assets: Res<Assets<Gltf>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
    gltf_nodes: Res<Assets<GltfNode>>,
    meshes: Res<Assets<Mesh>>,
) {
    for (entity, loader) in loader_query.iter() {
        let Some(gltf) = gltf_assets.get(&loader.handle) else {
            continue; // Still loading
        };

        let mut regions = Vec::new();

        for (node_name, node_handle) in &gltf.named_nodes {
            let Some(gltf_node) = gltf_nodes.get(node_handle) else {
                warn!("GltfNode not found for hitbox node '{}'", node_name);
                continue;
            };

            let Some(gltf_mesh_handle) = &gltf_node.mesh else {
                continue; // Skip empty/grouping nodes
            };

            let Some(gltf_mesh) = gltf_meshes.get(gltf_mesh_handle) else {
                warn!("GltfMesh not found for hitbox node '{}'", node_name);
                continue;
            };

            // Parse custom properties from glTF extras
            let properties = parse_extras(&gltf_node.extras);
            let base_damage = properties
                .get("base_damage")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as f32;

            for primitive in &gltf_mesh.primitives {
                let Some(mesh) = meshes.get(&primitive.mesh) else {
                    warn!("Mesh not found for hitbox node '{}'", node_name);
                    continue;
                };

                let Some(vertices) = extract_mesh_vertices(mesh) else {
                    warn!("Could not extract vertices from hitbox node '{}'", node_name);
                    continue;
                };
                let Some(indices) = extract_mesh_indices(mesh) else {
                    warn!("Could not extract indices from hitbox node '{}'", node_name);
                    continue;
                };

                let collider = Collider::trimesh(vertices, indices);

                regions.push(HitboxRegionData {
                    name: node_name.to_string(),
                    base_damage,
                    collider,
                    transform: gltf_node.transform,
                });

                info!(
                    "Parsed hitbox region '{}' with base_damage={}",
                    node_name, base_damage
                );
            }
        }

        info!("Player hitbox loaded with {} regions", regions.len());
        commands.insert_resource(PlayerHitboxData { regions });
        commands.entity(entity).despawn();
    }
}

/// Attach hitbox collider children to a player entity.
/// Call this from the server's `handle_connected` when spawning a new player.
pub fn attach_hitbox_to_player(commands: &mut Commands, player_entity: Entity, hitbox_data: &PlayerHitboxData) {
    commands.entity(player_entity).with_children(|parent| {
        for region in &hitbox_data.regions {
            parent.spawn((
                region.collider.clone(),
                Sensor,
                region.transform,
                GlobalTransform::default(),
                PlayerHitboxMarker,
                HitboxRegion {
                    name: region.name.clone(),
                    base_damage: region.base_damage,
                },
                Name::new(format!("Hitbox: {}", region.name)),
            ));
        }
    });
}
