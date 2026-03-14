use bevy::prelude::*;
use std::collections::HashMap;

use super::types::{ActionType, DynamicTween, EasingType, TweenType};

/// System that ticks all active mesh tweens and applies computed values to transforms.
pub fn tick_mesh_tweens(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut DynamicTween, &mut Transform)>,
) {
    let dt = time.delta_secs();

    for (entity, mut tween, mut transform) in query.iter_mut() {
        tween.elapsed += dt;
        let t = (tween.elapsed / tween.duration).clamp(0.0, 1.0);
        let eased = tween.easing.apply(t);

        match tween.tween_type {
            TweenType::Translation => {
                transform.translation = tween.start.lerp(tween.target, eased);
            }
            TweenType::Rotation => {
                // Start and target are stored as euler angles in degrees.
                // Convert to radians, create quaternions, and slerp.
                let start_rad = Vec3::new(
                    tween.start.x.to_radians(),
                    tween.start.y.to_radians(),
                    tween.start.z.to_radians(),
                );
                let target_rad = Vec3::new(
                    tween.target.x.to_radians(),
                    tween.target.y.to_radians(),
                    tween.target.z.to_radians(),
                );
                let start_quat = Quat::from_euler(
                    EulerRot::XYZ,
                    start_rad.x,
                    start_rad.y,
                    start_rad.z,
                );
                let target_quat = Quat::from_euler(
                    EulerRot::XYZ,
                    target_rad.x,
                    target_rad.y,
                    target_rad.z,
                );
                transform.rotation = start_quat.slerp(target_quat, eased);
            }
            TweenType::Scale => {
                transform.scale = tween.start.lerp(tween.target, eased);
            }
        }

        if t >= 1.0 {
            commands.entity(entity).remove::<DynamicTween>();
        }
    }
}

/// Parse a Vec3 from action params. Expects a JSON array like `[x, y, z]`.
fn parse_vec3(params: &HashMap<String, serde_json::Value>, key: &str) -> Option<Vec3> {
    let arr = params.get(key)?.as_array()?;
    if arr.len() >= 3 {
        Some(Vec3::new(
            arr[0].as_f64()? as f32,
            arr[1].as_f64()? as f32,
            arr[2].as_f64()? as f32,
        ))
    } else {
        None
    }
}

/// Start a tween from an action definition. Called by the client visual action system.
pub fn start_tween_from_action(
    commands: &mut Commands,
    entity: Entity,
    transform: &Transform,
    action_type: &ActionType,
    params: &HashMap<String, serde_json::Value>,
) {
    let duration = params
        .get("duration")
        .and_then(|v| v.as_f64())
        .unwrap_or(1.0) as f32;
    let easing = params
        .get("easing")
        .and_then(|v| v.as_str())
        .map(EasingType::from_str)
        .unwrap_or_default();

    match action_type {
        ActionType::MoveTo => {
            if let Some(offset) = parse_vec3(params, "offset") {
                commands.entity(entity).insert(DynamicTween {
                    tween_type: TweenType::Translation,
                    start: transform.translation,
                    target: offset,
                    duration,
                    elapsed: 0.0,
                    easing,
                });
            }
        }
        ActionType::RotateTo => {
            if let Some(rotation) = parse_vec3(params, "rotation") {
                // Extract current euler angles in degrees for the start value
                let (x, y, z) = transform.rotation.to_euler(EulerRot::XYZ);
                let current_degrees = Vec3::new(
                    x.to_degrees(),
                    y.to_degrees(),
                    z.to_degrees(),
                );
                commands.entity(entity).insert(DynamicTween {
                    tween_type: TweenType::Rotation,
                    start: current_degrees,
                    target: rotation,
                    duration,
                    elapsed: 0.0,
                    easing,
                });
            }
        }
        ActionType::ScaleTo => {
            if let Some(scale) = parse_vec3(params, "scale") {
                commands.entity(entity).insert(DynamicTween {
                    tween_type: TweenType::Scale,
                    start: transform.scale,
                    target: scale,
                    duration,
                    elapsed: 0.0,
                    easing,
                });
            }
        }
        _ => {}
    }
}
