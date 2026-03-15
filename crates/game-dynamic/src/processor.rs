use avian3d::prelude::*;
use bevy::gltf::{Gltf, GltfMesh, GltfNode};
use bevy::mesh::Mesh;
use bevy::prelude::*;

use super::debug::{DynamicDebugMesh, DynamicDebugSettings};
use super::types::DynamicObjectsConfig;
use super::types::*;
use super::{DynamicLoader, DynamicPluginConfig};
use game_core::world::{extract_mesh_indices, extract_mesh_vertices};

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
                    entity_type: node_config.node_type.clone(),
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

            // Spawn light components for light-type nodes
            if node_config.node_type == EntityType::Light {
                if let Some(ref light_info) = node_config.light_info {
                    let color = Color::linear_rgb(
                        light_info.color[0],
                        light_info.color[1],
                        light_info.color[2],
                    );
                    let scales = &objects_config.light_intensity_scales;
                    let raw_intensity = light_info.intensity;

                    match light_info.light_type.as_str() {
                        "point" => {
                            entity_commands.insert(PointLight {
                                color,
                                intensity: raw_intensity * scales.point,
                                shadows_enabled: true,
                                ..default()
                            });
                        }
                        "spot" => {
                            entity_commands.insert(SpotLight {
                                color,
                                intensity: raw_intensity * scales.spot,
                                shadows_enabled: true,
                                ..default()
                            });
                        }
                        "directional" => {
                            entity_commands.insert(DirectionalLight {
                                color,
                                illuminance: raw_intensity * scales.directional,
                                shadows_enabled: true,
                                ..default()
                            });
                        }
                        other => {
                            warn!(
                                "Unknown light_type '{}' for dynamic node '{}'",
                                other, node_name
                            );
                        }
                    }

                    // Insert ActiveLightEffects for procedural light effect system
                    entity_commands.insert(ActiveLightEffects::default());
                } else {
                    warn!(
                        "Dynamic node '{}' is EntityType::Light but has no light_info",
                        node_name
                    );
                }
            }

            // Attach timer/delay components for time-based triggers
            for (trigger_index, trigger_def) in node_config.triggers.iter().enumerate() {
                match trigger_def.trigger_type {
                    TriggerType::OnTimer => {
                        let interval = trigger_def
                            .params
                            .get("interval")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(1.0) as f32;
                        let repeat = trigger_def
                            .params
                            .get("repeat")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(true);
                        let mode = if repeat {
                            bevy::time::TimerMode::Repeating
                        } else {
                            bevy::time::TimerMode::Once
                        };
                        entity_commands.insert(DynamicTimer {
                            timer: bevy::time::Timer::from_seconds(interval, mode),
                            trigger_index,
                        });
                    }
                    TriggerType::OnDelay => {
                        let delay = trigger_def
                            .params
                            .get("delay")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(1.0) as f32;
                        entity_commands.insert(DynamicDelay {
                            timer: bevy::time::Timer::from_seconds(
                                delay,
                                bevy::time::TimerMode::Once,
                            ),
                            trigger_index,
                        });
                    }
                    _ => {}
                }
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
