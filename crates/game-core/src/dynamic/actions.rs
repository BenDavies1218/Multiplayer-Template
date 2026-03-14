use bevy::prelude::*;

use super::events::DynamicActionEvent;
use super::types::*;

/// Execute state-mutating actions (server-only).
///
/// Handles: `toggle_state`, `collect`, `enable`, `disable`.
/// State changes on `DynamicState` are automatically replicated to clients via Lightyear.
pub fn execute_state_actions(
    mut action_events: MessageReader<DynamicActionEvent>,
    mut state_query: Query<(&mut DynamicState, &DynamicObject)>,
    mut enabled_query: Query<&mut DynamicEnabled>,
    registry: Option<Res<DynamicObjectRegistry>>,
    mut commands: Commands,
) {
    for event in action_events.read() {
        match event.action.action_type {
            ActionType::ToggleState => {
                if let Ok((mut state, obj)) = state_query.get_mut(event.object)
                    && state.togglable
                {
                    let new_state = if state.current == "open" {
                        "closed"
                    } else if state.current == "closed" {
                        "open"
                    } else if state.current == "on" {
                        "off"
                    } else if state.current == "off" {
                        "on"
                    } else if state.current == "active" {
                        "idle"
                    } else {
                        "active"
                    };
                    info!(
                        "Dynamic '{}' state: {} -> {}",
                        obj.object_id, state.current, new_state
                    );
                    state.current = new_state.to_string();
                }
            }
            ActionType::Collect => {
                if let Ok((_, obj)) = state_query.get(event.object) {
                    info!(
                        "Dynamic '{}' collected by {:?}",
                        obj.object_id, event.source
                    );
                    // TODO: Grant effect to player based on action params
                    commands.entity(event.object).despawn();
                }
            }
            ActionType::Enable => {
                let target = resolve_target(&event.action, &registry);
                let target_entity = target.unwrap_or(event.object);
                if let Ok(mut enabled) = enabled_query.get_mut(target_entity) {
                    enabled.0 = true;
                    info!("Dynamic object {:?} enabled", target_entity);
                }
            }
            ActionType::Disable => {
                let target = resolve_target(&event.action, &registry);
                let target_entity = target.unwrap_or(event.object);
                if let Ok(mut enabled) = enabled_query.get_mut(target_entity) {
                    enabled.0 = false;
                    info!("Dynamic object {:?} disabled", target_entity);
                }
            }
            // Other action types are handled by client-side systems
            _ => {}
        }
    }
}

/// Resolve a `target` param from an action to an entity via the registry.
fn resolve_target(
    action: &ActionDef,
    registry: &Option<Res<DynamicObjectRegistry>>,
) -> Option<Entity> {
    let target_name = action.params.get("target")?.as_str()?;
    registry.as_ref()?.get(target_name)
}
