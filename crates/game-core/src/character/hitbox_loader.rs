use super::components::{CharacterHitboxMarker, HitboxRegion};
use crate::core_config::{GameCoreConfig, HitboxShape};
use crate::world::parse_extras;
use avian3d::prelude::*;
use bevy::gltf::{Gltf, GltfNode};
use bevy::prelude::*;

/// Temporary component placed on a loader entity while the player model GLB loads.
/// Removed after the GLB is processed into `CharacterHitboxData`.
#[derive(Component, Debug)]
pub struct CharacterModelLoader {
    pub handle: Handle<Gltf>,
}

/// A single parsed hitbox region ready to be attached to character entities.
#[derive(Clone, Debug)]
pub struct HitboxRegionData {
    pub name: String,
    pub base_damage: f32,
    pub collider: Collider,
    pub transform: Transform,
}

/// Resource holding the parsed hitbox regions.
/// Inserted once the player model GLB has been processed.
/// Used by the server when spawning characters to attach hitbox children.
#[derive(Resource, Debug, Clone)]
pub struct CharacterHitboxData {
    pub regions: Vec<HitboxRegionData>,
}

/// System that processes the player model GLB once it's loaded.
/// Scans for nodes with a `hitbox_region` custom property set in Blender.
/// Creates simple shape colliders (from config) positioned at the tagged node transforms.
pub fn process_character_model_hitboxes(
    mut commands: Commands,
    loader_query: Query<(Entity, &CharacterModelLoader)>,
    gltf_assets: Res<Assets<Gltf>>,
    gltf_nodes: Res<Assets<GltfNode>>,
    config: Res<GameCoreConfig>,
) {
    for (entity, loader) in loader_query.iter() {
        let Some(gltf) = gltf_assets.get(&loader.handle) else {
            continue; // Still loading
        };

        let mut regions = Vec::new();

        for (node_name, node_handle) in &gltf.named_nodes {
            let Some(gltf_node) = gltf_nodes.get(node_handle) else {
                warn!("GltfNode not found for node '{}'", node_name);
                continue;
            };

            // Check for hitbox_region custom property in glTF extras
            let properties = parse_extras(&gltf_node.extras);
            let Some(region_name) = properties
                .get("hitbox_region")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
            else {
                continue; // Not a hitbox node
            };

            // Look up region config for damage and shape
            let Some(region_config) = config.character.hitbox_regions.get(&region_name) else {
                warn!(
                    "Node '{}' has hitbox_region='{}' but no matching config entry — skipping",
                    node_name, region_name
                );
                continue;
            };

            let collider = match &region_config.shape {
                HitboxShape::Capsule {
                    radius,
                    half_height,
                } => Collider::capsule(*radius, *half_height * 2.0),
                HitboxShape::Box { half_extents } => {
                    Collider::cuboid(half_extents[0], half_extents[1], half_extents[2])
                }
            };

            regions.push(HitboxRegionData {
                name: region_name.clone(),
                base_damage: region_config.damage,
                collider,
                transform: gltf_node.transform,
            });

            info!(
                "Parsed hitbox region '{}' from node '{}' with damage={}",
                region_name, node_name, region_config.damage
            );
        }

        info!(
            "Character model processed with {} hitbox regions",
            regions.len()
        );
        commands.insert_resource(CharacterHitboxData { regions });
        commands.entity(entity).despawn();
    }
}

/// Attach hitbox collider children to a character entity.
/// Call this from the server's `handle_connected` when spawning a new character.
pub fn attach_hitbox_to_character(
    commands: &mut Commands,
    character_entity: Entity,
    hitbox_data: &CharacterHitboxData,
) {
    commands.entity(character_entity).with_children(|parent| {
        for region in &hitbox_data.regions {
            parent.spawn((
                region.collider.clone(),
                Sensor,
                region.transform,
                GlobalTransform::default(),
                CharacterHitboxMarker,
                HitboxRegion {
                    name: region.name.clone(),
                    base_damage: region.base_damage,
                },
                Name::new(format!("Hitbox: {}", region.name)),
            ));
        }
    });
}
