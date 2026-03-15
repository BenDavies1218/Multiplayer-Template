use bevy::prelude::*;
use game_dynamic::{
    light_effects, mesh_effects, ActionType, ActiveLightEffects, DynamicActionEvent,
    DynamicObject, DynamicObjectRegistry, DynamicState,
};

/// Client-side plugin for visual action execution on dynamic objects.
///
/// Handles animations, light changes, text display, and reacts to
/// replicated `DynamicState` changes from the server.
pub struct DynamicRenderingPlugin;

impl Plugin for DynamicRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (execute_visual_actions, sync_dynamic_state_visuals));
    }
}

/// Execute client-side visual actions from `DynamicActionEvent`.
#[allow(clippy::too_many_arguments)]
fn execute_visual_actions(
    mut commands: Commands,
    mut action_events: MessageReader<DynamicActionEvent>,
    _registry: Option<Res<DynamicObjectRegistry>>,
    name_query: Query<&Name>,
    mut point_lights: Query<(&Name, &mut PointLight)>,
    mut spot_lights: Query<(&Name, &mut SpotLight)>,
    mut effects_query: Query<&mut ActiveLightEffects>,
    transform_query: Query<&Transform>,
) {
    for event in action_events.read() {
        match event.action.action_type {
            ActionType::SetLightIntensity => {
                let target_name = event.action.params.get("target").and_then(|v| v.as_str());
                let intensity = event
                    .action
                    .params
                    .get("intensity")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(1.0) as f32;

                if let Some(target) = target_name {
                    // Find light by name
                    for (name, mut light) in point_lights.iter_mut() {
                        if name.as_str() == target {
                            info!(
                                "Setting point light '{}' intensity to {}",
                                target, intensity
                            );
                            light.intensity = intensity;
                        }
                    }
                    for (name, mut light) in spot_lights.iter_mut() {
                        if name.as_str() == target {
                            info!("Setting spot light '{}' intensity to {}", target, intensity);
                            light.intensity = intensity;
                        }
                    }
                }
            }
            ActionType::SetLightColor => {
                let target_name = event.action.params.get("target").and_then(|v| v.as_str());
                let color_hex = event
                    .action
                    .params
                    .get("color")
                    .and_then(|v| v.as_str())
                    .unwrap_or("#ffffff");

                let color = parse_hex_color(color_hex);

                if let Some(target) = target_name {
                    for (name, mut light) in point_lights.iter_mut() {
                        if name.as_str() == target {
                            info!("Setting point light '{}' color to {}", target, color_hex);
                            light.color = color;
                        }
                    }
                    for (name, mut light) in spot_lights.iter_mut() {
                        if name.as_str() == target {
                            info!("Setting spot light '{}' color to {}", target, color_hex);
                            light.color = color;
                        }
                    }
                }
            }
            ActionType::PlayAnimation => {
                let animation_name = event
                    .action
                    .params
                    .get("animation")
                    .and_then(|v| v.as_str())
                    .unwrap_or("default");

                if let Ok(name) = name_query.get(event.object) {
                    info!(
                        "Play animation '{}' on dynamic object '{}'",
                        animation_name, name
                    );
                }
                // TODO: Look up AnimationPlayer on the entity and play the named clip
            }
            ActionType::StopAnimation => {
                if let Ok(name) = name_query.get(event.object) {
                    info!("Stop animation on dynamic object '{}'", name);
                }
                // TODO: Stop AnimationPlayer on the entity
            }
            ActionType::DisplayText => {
                let text = event
                    .action
                    .params
                    .get("text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                info!("Display text: '{}'", text);
                // TODO: Spawn UI text overlay for the triggering player
            }
            ActionType::HideText => {
                info!("Hide text");
                // TODO: Despawn UI text overlay
            }
            ActionType::PlaySound => {
                let sound = event
                    .action
                    .params
                    .get("sound")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                info!("Play sound: '{}'", sound);
                // TODO: Play audio clip
            }
            ActionType::StartLightEffect => {
                if let Ok(mut effects) = effects_query.get_mut(event.object) {
                    light_effects::apply_start_light_effect(&mut effects, &event.action.params);
                }
            }
            ActionType::StopLightEffect => {
                if let Ok(mut effects) = effects_query.get_mut(event.object) {
                    light_effects::apply_stop_light_effect(&mut effects, &event.action.params);
                }
            }
            ActionType::MoveTo | ActionType::RotateTo | ActionType::ScaleTo => {
                if let Ok(transform) = transform_query.get(event.object) {
                    mesh_effects::start_tween_from_action(
                        &mut commands,
                        event.object,
                        transform,
                        &event.action.action_type,
                        &event.action.params,
                    );
                }
            }
            // State actions (toggle, collect, enable, disable) are handled server-side
            _ => {}
        }
    }
}

/// React to replicated `DynamicState` changes from the server.
fn sync_dynamic_state_visuals(
    changed_query: Query<(&DynamicObject, &DynamicState), Changed<DynamicState>>,
) {
    for (obj, state) in changed_query.iter() {
        info!(
            "Dynamic '{}' state changed to '{}'",
            obj.object_id, state.current
        );
        // TODO: Trigger visual transitions based on state (e.g. door open/close animation)
    }
}

/// Parse a hex color string like "#ff0000" into a Bevy Color.
fn parse_hex_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() >= 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);
        Color::srgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
    } else {
        Color::WHITE
    }
}
