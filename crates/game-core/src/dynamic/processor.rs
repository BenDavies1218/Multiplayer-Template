use avian3d::prelude::*;
use bevy::gltf::{Gltf, GltfMesh, GltfNode};
use bevy::mesh::Mesh;
use bevy::prelude::*;

use super::debug::{DynamicDebugMesh, DynamicDebugSettings};
use super::types::DynamicObjectsConfig;
use super::types::*;
use super::{DynamicLoader, DynamicPluginConfig};
use crate::world::{extract_mesh_indices, extract_mesh_vertices};

/// Process dynamic object nodes from the loaded GLB.
///
/// Iterates all named nodes, looks up behavior from `DynamicObjectsConfig`,
/// and spawns entities with the appropriate components.
#[allow(clippy::too_many_arguments)]
pub fn process_dynamic_objects(
    mut commands: Commands,
    loader_query: Query<(Entity, &DynamicLoader)>,
    gltf_assets: Res<Assets<Gltf>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
    gltf_nodes: Res<Assets<GltfNode>>,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    mut materials: Option<ResMut<Assets<StandardMaterial>>>,
    objects_config: Res<DynamicObjectsConfig>,
    plugin_config: Res<DynamicPluginConfig>,
    debug_settings: Option<Res<DynamicDebugSettings>>,
) {
    for (entity, loader) in loader_query.iter() {
        let Some(gltf) = gltf_assets.get(&loader.handle) else {
            continue; // Still loading
        };

        let mut registry = DynamicObjectRegistry::default();

        for (node_name, _) in &gltf.named_nodes {
            info!("  Dynamic node available: '{}'", node_name);
        }

        for (node_name, node_handle) in &gltf.named_nodes {
            let Some(gltf_node) = gltf_nodes.get(node_handle) else {
                warn!("GltfNode not found for dynamic node '{}'", node_name);
                continue;
            };

            // Look up this node in the config file
            let node_name_str: &str = node_name;
            let Some(node_config) = objects_config.nodes.get(node_name_str) else {
                continue; // Node not in config — skip silently
            };

            let node_transform = gltf_node.transform;
            let object_id = node_name.clone();

            // Build state component
            let state = match &node_config.state {
                Some(sc) => DynamicState {
                    current: sc.initial.clone(),
                    togglable: sc.toggle,
                },
                None => DynamicState {
                    current: "idle".to_string(),
                    togglable: false,
                },
            };

            let triggers = node_config.triggers.clone();

            // Extract interaction radius from any playerOnInteract trigger
            let interaction_radius = triggers.iter().find_map(|t| {
                if t.trigger_type == TriggerType::PlayerOnInteract {
                    t.params
                        .get("radius")
                        .and_then(|v| v.as_f64())
                        .map(|r| r as f32)
                } else {
                    None
                }
            });

            // Check if any trigger needs a sensor collider (enter/exit)
            let needs_sensor = triggers.iter().any(|t| {
                matches!(
                    t.trigger_type,
                    TriggerType::PlayerOnEnter | TriggerType::PlayerOnExit
                )
            });

            let num_triggers = triggers.len();

            // Spawn the entity
            let mut entity_commands = commands.spawn((
                DynamicObject {
                    object_type: node_name.to_string(),
                    object_id: object_id.to_string(),
                },
                state,
                DynamicBehavior { triggers },
                DynamicEnabled(true),
                node_transform,
                GlobalTransform::default(),
                Visibility::default(),
                Name::new(format!("Dynamic: {}", node_name)),
            ));

            if let Some(radius) = interaction_radius {
                entity_commands.insert(InteractionRadius(radius));
            }

            // Extract mesh data for visuals, collider, and/or debug visualization
            let needs_mesh =
                needs_sensor || plugin_config.enable_visuals || plugin_config.enable_debug;
            if needs_mesh {
                if let Some(gltf_mesh_handle) = &gltf_node.mesh {
                    if let Some(gltf_mesh) = gltf_meshes.get(gltf_mesh_handle) {
                        for primitive in &gltf_mesh.primitives {
                            let mesh = meshes.as_ref().and_then(|m| m.get(&primitive.mesh));
                            let Some(mesh) = mesh else {
                                warn!("Mesh not found for dynamic node '{}'", node_name);
                                continue;
                            };

                            // Clone mesh upfront to avoid borrow conflicts
                            let mesh_clone = mesh.clone();

                            // Create sensor collider if needed
                            if needs_sensor
                                && let (Some(vertices), Some(indices)) =
                                    (extract_mesh_vertices(mesh), extract_mesh_indices(mesh))
                            {
                                entity_commands.insert((
                                    Collider::trimesh(vertices, indices),
                                    Sensor,
                                    CollisionEventsEnabled,
                                    RigidBody::Static,
                                ));
                            }

                            // Spawn visual mesh with original material (client)
                            if plugin_config.enable_visuals {
                                let visual_material = primitive
                                    .material
                                    .clone()
                                    .unwrap_or_else(|| {
                                        materials
                                            .as_mut()
                                            .map(|m| m.add(StandardMaterial::default()))
                                            .unwrap()
                                    });

                                entity_commands.with_children(|parent| {
                                    parent.spawn((
                                        Mesh3d(primitive.mesh.clone()),
                                        MeshMaterial3d(visual_material),
                                        Transform::default(),
                                        GlobalTransform::default(),
                                        Name::new(format!("Visual: {}", node_name)),
                                    ));
                                });
                            }

                            // Add debug visualization mesh
                            if plugin_config.enable_debug
                                && let (Some(settings), Some(mesh_res), Some(mat_res)) =
                                    (&debug_settings, &mut meshes, &mut materials)
                            {
                                let debug_mesh_handle = mesh_res.add(mesh_clone.clone());
                                let debug_material = mat_res.add(StandardMaterial {
                                    base_color: settings.dynamic_object_color,
                                    alpha_mode: AlphaMode::Blend,
                                    double_sided: true,
                                    cull_mode: None,
                                    ..default()
                                });

                                entity_commands.with_children(|parent| {
                                    parent.spawn((
                                        Mesh3d(debug_mesh_handle),
                                        MeshMaterial3d(debug_material),
                                        Transform::default(),
                                        GlobalTransform::default(),
                                        Visibility::Hidden,
                                        DynamicDebugMesh,
                                        Name::new(format!("DebugDynamic: {}", node_name)),
                                    ));
                                });
                            }
                        }
                    }
                } else if needs_sensor {
                    warn!(
                        "Dynamic node '{}' needs sensor collider but has no mesh",
                        node_name
                    );
                }
            }

            registry
                .by_id
                .insert(object_id.to_string(), entity_commands.id());

            info!(
                "Created dynamic object '{}' (triggers={}, has_sensor={})",
                node_name, num_triggers, needs_sensor,
            );
        }

        info!("Registered {} dynamic objects", registry.by_id.len());
        commands.insert_resource(registry);
        commands.entity(entity).despawn();
    }
}
