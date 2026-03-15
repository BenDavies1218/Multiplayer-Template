use crate::client_config::GameClientConfig;
use bevy::prelude::*;
use game_camera::GameCamera;
use game_core::{GameSimulationConfig, character::CharacterModelId};
use game_networking::protocol::{CharacterMarker, ColorComponent};
use lightyear::prelude::*;
use std::collections::HashMap;

/// Plugin for character model rendering.
///
/// Handles:
/// - Preloading character model assets (third-person + POV)
/// - Attaching third-person models to remote players
/// - Attaching POV models to the local player's camera
pub struct CharacterRenderingPlugin;

impl Plugin for CharacterRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, preload_character_models);
        app.add_systems(Update, add_character_cosmetics);
    }
}

/// Holds preloaded character model scene handles.
#[derive(Resource)]
struct CharacterModelAssets {
    /// Third-person models keyed by character model ID
    third_person: HashMap<String, Handle<Scene>>,
    /// POV models keyed by character model ID, then by variant name ("empty", weapon names)
    pov: HashMap<String, HashMap<String, Handle<Scene>>>,
}

/// Marker for the POV model entity attached as child of the camera.
#[derive(Component)]
struct PovModel;

fn preload_character_models(
    mut commands: Commands,
    client_config: Res<GameClientConfig>,
    asset_server: Res<AssetServer>,
) {
    let mut third_person = HashMap::new();
    let mut pov = HashMap::new();

    for (id, model_set) in &client_config.character.model_catalog {
        // Third-person model
        let tp_handle = asset_server.load(format!("{}#Scene0", model_set.player));
        info!(
            "Preloading character third-person model '{}' from {}",
            id, model_set.player
        );
        third_person.insert(id.clone(), tp_handle);

        // POV empty hands
        let mut pov_variants = HashMap::new();
        let pov_handle = asset_server.load(format!("{}#Scene0", model_set.pov_empty));
        info!(
            "Preloading character POV model '{}' from {}",
            id, model_set.pov_empty
        );
        pov_variants.insert("empty".to_string(), pov_handle);

        // POV weapon models
        for (weapon_name, path) in &model_set.pov_weapons {
            let h = asset_server.load(format!("{}#Scene0", path));
            info!(
                "Preloading character POV weapon '{}:{}' from {}",
                id, weapon_name, path
            );
            pov_variants.insert(weapon_name.clone(), h);
        }

        pov.insert(id.clone(), pov_variants);
    }

    commands.insert_resource(CharacterModelAssets { third_person, pov });
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn add_character_cosmetics(
    mut commands: Commands,
    character_query: Query<
        (
            Entity,
            &ColorComponent,
            Option<&CharacterModelId>,
            Has<Controlled>,
        ),
        (
            Or<(Added<Predicted>, Added<Replicate>, Added<Interpolated>)>,
            With<CharacterMarker>,
        ),
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    sim_config: Res<GameSimulationConfig>,
    character_models: Option<Res<CharacterModelAssets>>,
    camera_query: Query<Entity, With<GameCamera>>,
    pov_query: Query<Entity, With<PovModel>>,
) {
    for (entity, color, model_id, is_local_player) in &character_query {
        let model_key = model_id.map(|m| m.0.as_str()).unwrap_or("default");

        if is_local_player {
            // Local player: attach POV model as child of camera
            // Skip if POV model already exists
            if !pov_query.is_empty() {
                continue;
            }

            let pov_scene = character_models
                .as_ref()
                .and_then(|cm| cm.pov.get(model_key))
                .and_then(|variants| variants.get("empty"))
                .cloned();

            if let Some(scene_handle) = pov_scene {
                if let Ok(camera_entity) = camera_query.single() {
                    commands.entity(camera_entity).with_children(|parent| {
                        parent.spawn((
                            SceneRoot(scene_handle),
                            Transform::from_xyz(0.0, -0.3, -0.5),
                            PovModel,
                            Name::new("POV Model"),
                        ));
                    });
                    info!("Attached POV model '{}' to camera", model_key);
                }
            } else {
                warn!("POV model '{}' not found in catalog", model_key);
            }
            continue;
        }

        // Remote player: attach third-person model
        let attached_model = character_models
            .as_ref()
            .and_then(|cm| cm.third_person.get(model_key))
            .cloned();

        if let Some(scene_handle) = attached_model {
            commands.entity(entity).insert(SceneRoot(scene_handle));
            info!(
                "Attached character model '{}' to entity {:?}",
                model_key, entity
            );
        } else {
            // Fallback to capsule if model not found
            commands.entity(entity).insert((
                Mesh3d(meshes.add(Capsule3d::new(
                    sim_config.character.capsule_radius,
                    sim_config.character.capsule_height,
                ))),
                MeshMaterial3d(materials.add(color.0)),
            ));
            warn!(
                "Character model '{}' not found in catalog, using capsule fallback",
                model_key
            );
        }
    }
}
