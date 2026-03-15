use bevy::prelude::*;
use std::collections::HashMap;
use std::f32::consts::TAU;

use super::types::{
    ActiveLightEffects, LightColorEffectInstance, LightColorEffectType, LightEffectInstance,
    LightEffectType,
};

/// System that ticks all active light effects and applies computed values to light components.
#[allow(clippy::type_complexity)]
pub fn tick_light_effects(
    time: Res<Time>,
    mut query: Query<(
        &mut ActiveLightEffects,
        Option<&mut PointLight>,
        Option<&mut SpotLight>,
        Option<&mut DirectionalLight>,
    )>,
) {
    let dt = time.delta_secs();

    for (mut effects, mut point_light, mut spot_light, mut dir_light) in query.iter_mut() {
        // Tick intensity effect
        if let Some(ref mut eff) = effects.intensity_effect {
            eff.elapsed += dt;
            let intensity = compute_intensity(eff);

            if let Some(ref mut light) = point_light {
                light.intensity = intensity;
            } else if let Some(ref mut light) = spot_light {
                light.intensity = intensity;
            } else if let Some(ref mut light) = dir_light {
                light.illuminance = intensity;
            }
        }

        // Tick color effect
        if let Some(ref mut eff) = effects.color_effect {
            eff.elapsed += dt;
            let color = compute_color(eff);

            if let Some(ref mut light) = point_light {
                light.color = color;
            } else if let Some(ref mut light) = spot_light {
                light.color = color;
            } else if let Some(ref mut light) = dir_light {
                light.color = color;
            }
        }
    }
}

/// Compute intensity from an active intensity effect.
fn compute_intensity(effect: &LightEffectInstance) -> f32 {
    let min = effect.min;
    let max = effect.max;

    match effect.effect_type {
        LightEffectType::Fixed => max,
        LightEffectType::Pulse => {
            let t = effect.elapsed * effect.speed;
            min + (max - min) * ((t * TAU).sin() + 1.0) * 0.5
        }
        LightEffectType::Flicker => {
            let t = effect.elapsed * effect.speed;
            // Multi-frequency noise approximation
            let noise = (t * 1.0).sin() * 0.5 + (t * 2.3).sin() * 0.3 + (t * 5.7).sin() * 0.2;
            // noise range is roughly [-1, 1], map to [min, max]
            let normalized = (noise + 1.0) * 0.5;
            min + (max - min) * normalized
        }
    }
}

/// Compute color from an active color effect.
fn compute_color(effect: &LightColorEffectInstance) -> Color {
    match &effect.effect_type {
        LightColorEffectType::Fixed { color } => Color::linear_rgb(color[0], color[1], color[2]),
        LightColorEffectType::Cycle { colors, speed } => {
            if colors.is_empty() {
                return Color::WHITE;
            }
            if colors.len() == 1 {
                let c = &colors[0];
                return Color::linear_rgb(c[0], c[1], c[2]);
            }

            let t = (effect.elapsed * speed) % (colors.len() as f32);
            let idx = t.floor() as usize;
            let frac = t - t.floor();

            let a = &colors[idx % colors.len()];
            let b = &colors[(idx + 1) % colors.len()];

            Color::linear_rgb(
                a[0] + (b[0] - a[0]) * frac,
                a[1] + (b[1] - a[1]) * frac,
                a[2] + (b[2] - a[2]) * frac,
            )
        }
        LightColorEffectType::Flicker { min, max, speed } => {
            let t = effect.elapsed * speed;
            let noise = (t * 1.0).sin() * 0.5 + (t * 2.3).sin() * 0.3 + (t * 5.7).sin() * 0.2;
            let normalized = (noise + 1.0) * 0.5;

            Color::linear_rgb(
                min[0] + (max[0] - min[0]) * normalized,
                min[1] + (max[1] - min[1]) * normalized,
                min[2] + (max[2] - min[2]) * normalized,
            )
        }
        LightColorEffectType::Pulse { min, max, speed } => {
            let t = effect.elapsed * speed;
            let factor = ((t * TAU).sin() + 1.0) * 0.5;

            Color::linear_rgb(
                min[0] + (max[0] - min[0]) * factor,
                min[1] + (max[1] - min[1]) * factor,
                min[2] + (max[2] - min[2]) * factor,
            )
        }
    }
}

/// Parse a color parameter from action params (expects an array of 3 floats).
fn parse_color_param(params: &HashMap<String, serde_json::Value>, key: &str) -> Option<[f32; 3]> {
    params.get(key).and_then(|v| {
        v.as_array().and_then(|arr| {
            if arr.len() >= 3 {
                Some([
                    arr[0].as_f64()? as f32,
                    arr[1].as_f64()? as f32,
                    arr[2].as_f64()? as f32,
                ])
            } else {
                None
            }
        })
    })
}

/// Parse a `colors` array parameter (array of [r,g,b] arrays).
fn parse_colors_array(params: &HashMap<String, serde_json::Value>) -> Vec<[f32; 3]> {
    params
        .get("colors")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    item.as_array().and_then(|c| {
                        if c.len() >= 3 {
                            Some([
                                c[0].as_f64()? as f32,
                                c[1].as_f64()? as f32,
                                c[2].as_f64()? as f32,
                            ])
                        } else {
                            None
                        }
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Apply a `StartLightEffect` action to the entity's active effects.
pub fn apply_start_light_effect(
    effects: &mut ActiveLightEffects,
    params: &HashMap<String, serde_json::Value>,
) {
    let effect = params
        .get("effect")
        .and_then(|v| v.as_str())
        .unwrap_or("fixed");
    let channel = params.get("channel").and_then(|v| v.as_str()).unwrap_or("");
    let min = params.get("min").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
    let max = params.get("max").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;
    let speed = params.get("speed").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;

    match effect {
        "flicker" => {
            if channel == "color" {
                let min_color = parse_color_param(params, "min_color").unwrap_or([0.0, 0.0, 0.0]);
                let max_color = parse_color_param(params, "max_color").unwrap_or([1.0, 1.0, 1.0]);
                effects.color_effect = Some(LightColorEffectInstance {
                    effect_type: LightColorEffectType::Flicker {
                        min: min_color,
                        max: max_color,
                        speed,
                    },
                    elapsed: 0.0,
                });
            } else {
                // Default to intensity channel
                effects.intensity_effect = Some(LightEffectInstance {
                    effect_type: LightEffectType::Flicker,
                    min,
                    max,
                    speed,
                    elapsed: 0.0,
                });
            }
        }
        "pulse" => {
            if channel == "color" {
                let min_color = parse_color_param(params, "min_color").unwrap_or([0.0, 0.0, 0.0]);
                let max_color = parse_color_param(params, "max_color").unwrap_or([1.0, 1.0, 1.0]);
                effects.color_effect = Some(LightColorEffectInstance {
                    effect_type: LightColorEffectType::Pulse {
                        min: min_color,
                        max: max_color,
                        speed,
                    },
                    elapsed: 0.0,
                });
            } else {
                effects.intensity_effect = Some(LightEffectInstance {
                    effect_type: LightEffectType::Pulse,
                    min,
                    max,
                    speed,
                    elapsed: 0.0,
                });
            }
        }
        "cycle" => {
            let colors = parse_colors_array(params);
            effects.color_effect = Some(LightColorEffectInstance {
                effect_type: LightColorEffectType::Cycle { colors, speed },
                elapsed: 0.0,
            });
        }
        _ => {
            // Fixed can set intensity, color, or both
            if let Some(intensity) = params.get("intensity").and_then(|v| v.as_f64()) {
                effects.intensity_effect = Some(LightEffectInstance {
                    effect_type: LightEffectType::Fixed,
                    min: 0.0,
                    max: intensity as f32,
                    speed: 0.0,
                    elapsed: 0.0,
                });
            }
            if let Some(color) = parse_color_param(params, "color") {
                effects.color_effect = Some(LightColorEffectInstance {
                    effect_type: LightColorEffectType::Fixed { color },
                    elapsed: 0.0,
                });
            }
        }
    }
}

/// Apply a `StopLightEffect` action — clear active effects.
pub fn apply_stop_light_effect(
    effects: &mut ActiveLightEffects,
    params: &HashMap<String, serde_json::Value>,
) {
    let channel = params.get("channel").and_then(|v| v.as_str());

    match channel {
        Some("intensity") => {
            effects.intensity_effect = None;
        }
        Some("color") => {
            effects.color_effect = None;
        }
        _ => {
            // No channel specified or unknown — clear both
            effects.intensity_effect = None;
            effects.color_effect = None;
        }
    }
}
