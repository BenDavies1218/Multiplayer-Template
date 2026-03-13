use super::spawn_points::SpawnPoints;
use super::zone_debug::{ZoneDebugMesh, ZoneDebugSettings};
use super::zones::*;
use super::{ZoneLoader, ZonePluginConfig};
use crate::core_config::GameCoreConfig;
use crate::world::{extract_mesh_indices, extract_mesh_vertices, parse_extras};
use avian3d::prelude::*;
use bevy::gltf::{Gltf, GltfMesh, GltfNode};
use bevy::mesh::Mesh;
use bevy::prelude::*;

/// Process zone meshes from loaded zones GLB file.
///
/// Parses node names by prefix to determine zone type:
/// - `spawn_` -> SpawnPoint (transform only, no collider)
/// - `deathzone_` -> DeathZone (sensor trimesh collider)
/// - `damage_` -> DamageZone (sensor trimesh collider)
/// - `trigger_` -> TriggerZone (sensor trimesh collider)
///
/// Custom properties from Blender are read from GltfExtras JSON.
#[allow(clippy::too_many_arguments)]
pub fn process_zone_meshes(
    mut commands: Commands,
    zone_query: Query<(Entity, &ZoneLoader)>,
    gltf_assets: Res<Assets<Gltf>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
    gltf_nodes: Res<Assets<GltfNode>>,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    mut materials: Option<ResMut<Assets<StandardMaterial>>>,
    plugin_config: Res<ZonePluginConfig>,
    debug_settings: Option<Res<ZoneDebugSettings>>,
    config: Res<GameCoreConfig>,
) {
    for (entity, loader) in zone_query.iter() {
        let Some(gltf) = gltf_assets.get(&loader.handle) else {
            continue; // Still loading
        };

        let mut spawn_positions: Vec<(u32, Vec3)> = Vec::new();

        for (node_name, _) in &gltf.named_nodes {
            info!("  Zone node available: '{}'", node_name);
        }

        for (node_name, node_handle) in &gltf.named_nodes {
            let Some(gltf_node) = gltf_nodes.get(node_handle) else {
                warn!("GltfNode not found for zone {}", node_name);
                continue;
            };

            let node_transform = gltf_node.transform;
            let name_lower = node_name.to_lowercase();

            // Parse custom properties from glTF extras if available
            let properties = parse_extras(&gltf_node.extras);

            if name_lower.starts_with("spawn_") {
                let index = parse_index_from_name(&name_lower, "spawn_");
                spawn_positions.push((index, node_transform.translation));

                let mut spawn_commands = commands.spawn((
                    SpawnPoint { index },
                    ZoneProperties(properties),
                    node_transform,
                    GlobalTransform::default(),
                    Name::new(format!("SpawnPoint: {}", node_name)),
                ));

                // Add debug sphere for spawn points
                if plugin_config.enable_debug
                    && let (Some(settings), Some(mesh_res), Some(mat_res)) =
                        (&debug_settings, &mut meshes, &mut materials)
                {
                    let debug_mesh = mesh_res.add(Sphere::new(0.5));
                    let debug_material = mat_res.add(StandardMaterial {
                        base_color: settings.spawn_point_color,
                        alpha_mode: AlphaMode::Blend,
                        double_sided: true,
                        cull_mode: None,
                        ..default()
                    });

                    spawn_commands.with_children(|parent| {
                        parent.spawn((
                            Mesh3d(debug_mesh),
                            MeshMaterial3d(debug_material),
                            Transform::default(),
                            GlobalTransform::default(),
                            Visibility::Hidden,
                            ZoneDebugMesh,
                            Name::new(format!("DebugSpawn: {}", node_name)),
                        ));
                    });
                }

                info!(
                    "Created spawn point '{}' at {:?}",
                    node_name, node_transform.translation
                );
            } else if name_lower.starts_with("deathzone_")
                || name_lower.starts_with("damage_")
                || name_lower.starts_with("trigger_")
            {
                let Some(gltf_mesh_handle) = &gltf_node.mesh else {
                    warn!("Zone node '{}' has no mesh — skipping", node_name);
                    continue;
                };

                let Some(gltf_mesh) = gltf_meshes.get(gltf_mesh_handle) else {
                    warn!("GltfMesh not found for zone node {}", node_name);
                    continue;
                };

                for primitive in &gltf_mesh.primitives {
                    let mesh = meshes.as_ref().and_then(|m| m.get(&primitive.mesh));
                    let Some(mesh) = mesh else {
                        warn!("Mesh not found for zone node {}", node_name);
                        continue;
                    };

                    let Some(vertices) = extract_mesh_vertices(mesh) else {
                        warn!("Could not extract vertices from zone node {}", node_name);
                        continue;
                    };
                    let Some(indices) = extract_mesh_indices(mesh) else {
                        warn!("Could not extract indices from zone node {}", node_name);
                        continue;
                    };

                    // Clone mesh for debug visualization before consuming it
                    let mesh_clone_for_debug = mesh.clone();

                    let collider = Collider::trimesh(vertices, indices);

                    let mut entity_commands = commands.spawn((
                        collider,
                        Sensor,
                        CollisionEventsEnabled,
                        RigidBody::Static,
                        node_transform,
                        GlobalTransform::default(),
                        ZoneProperties(properties.clone()),
                        Name::new(format!("Zone: {}", node_name)),
                    ));

                    // Determine zone type and debug color
                    let debug_color;
                    if name_lower.starts_with("deathzone_") {
                        entity_commands.insert(DeathZone);
                        debug_color = debug_settings.as_ref().map(|s| s.death_zone_color);
                        info!("Created death zone '{}'", node_name);
                    } else if name_lower.starts_with("damage_") {
                        let damage = properties
                            .get("damage")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(config.zones.default_damage as f64)
                            as f32;
                        let interval = properties
                            .get("interval")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(config.zones.default_damage_interval as f64)
                            as f32;
                        entity_commands.insert(DamageZone { damage, interval });
                        debug_color = debug_settings.as_ref().map(|s| s.damage_zone_color);
                        info!(
                            "Created damage zone '{}' (damage={}, interval={})",
                            node_name, damage, interval
                        );
                    } else {
                        let event_name = properties
                            .get("event")
                            .and_then(|v| v.as_str())
                            .unwrap_or(node_name)
                            .to_string();
                        entity_commands.insert(TriggerZone { event_name });
                        debug_color = debug_settings.as_ref().map(|s| s.trigger_zone_color);
                        info!("Created trigger zone '{}'", node_name);
                    }

                    // Add debug mesh visualization
                    if plugin_config.enable_debug
                        && let (Some(color), Some(mesh_res), Some(mat_res)) =
                            (debug_color, &mut meshes, &mut materials)
                    {
                        let debug_mesh_handle = mesh_res.add(mesh_clone_for_debug);
                        let debug_material = mat_res.add(StandardMaterial {
                            base_color: color,
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
                                ZoneDebugMesh,
                                Name::new(format!("DebugZone: {}", node_name)),
                            ));
                        });
                    }
                }
            } else {
                warn!("Unknown zone node prefix for '{}' — skipping", node_name);
            }
        }

        // Build SpawnPoints resource sorted by index
        spawn_positions.sort_by_key(|(idx, _)| *idx);
        let points: Vec<Vec3> = spawn_positions.into_iter().map(|(_, pos)| pos).collect();
        info!("Registered {} spawn points", points.len());
        let default_pos = config.zones.default_spawn_position;
        commands.insert_resource(SpawnPoints::new(
            points,
            Vec3::new(default_pos[0], default_pos[1], default_pos[2]),
        ));

        commands.entity(entity).despawn();
    }
}

/// Parse a numeric index from a node name suffix (e.g., "spawn_02" -> 2)
fn parse_index_from_name(name: &str, prefix: &str) -> u32 {
    name.strip_prefix(prefix)
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0)
}
