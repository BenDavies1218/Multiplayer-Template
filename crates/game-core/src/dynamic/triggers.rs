use avian3d::prelude::*;
use bevy::prelude::*;

use super::events::{DynamicActionEvent, DynamicTriggerEvent};
use super::types::*;
use crate::character::CharacterMarker;

/// Detect when characters enter/exit dynamic object sensor colliders.
#[allow(clippy::type_complexity)]
pub fn detect_enter_exit_triggers(
    mut collision_start: MessageReader<CollisionStart>,
    mut collision_end: MessageReader<CollisionEnd>,
    character_query: Query<Entity, With<CharacterMarker>>,
    dynamic_query: Query<Entity, (With<DynamicObject>, With<DynamicEnabled>)>,
    enabled_query: Query<&DynamicEnabled>,
    mut trigger_events: MessageWriter<DynamicTriggerEvent>,
) {
    // Enter events
    for event in collision_start.read() {
        let (player, dynamic) = match resolve_collision_pair(
            event.collider1,
            event.collider2,
            event.body1,
            event.body2,
            &character_query,
            &dynamic_query,
        ) {
            Some(pair) => pair,
            None => continue,
        };

        if !is_enabled(&enabled_query, dynamic) {
            continue;
        }

        trigger_events.write(DynamicTriggerEvent {
            object: dynamic,
            trigger_type: TriggerType::PlayerOnEnter,
            source: player,
        });
    }

    // Exit events
    for event in collision_end.read() {
        let (player, dynamic) = match resolve_collision_pair(
            event.collider1,
            event.collider2,
            event.body1,
            event.body2,
            &character_query,
            &dynamic_query,
        ) {
            Some(pair) => pair,
            None => continue,
        };

        if !is_enabled(&enabled_query, dynamic) {
            continue;
        }

        trigger_events.write(DynamicTriggerEvent {
            object: dynamic,
            trigger_type: TriggerType::PlayerOnExit,
            source: player,
        });
    }
}

/// Detect playerOnInteract triggers (proximity + Interact input).
///
/// Note: The actual Interact input check requires `CharacterAction` from `game-networking`.
/// This system currently detects proximity only. The input check will be added when
/// this system is moved to or integrated with the networking layer.
pub fn detect_interact_triggers(
    character_query: Query<(Entity, &Position), With<CharacterMarker>>,
    dynamic_query: Query<
        (Entity, &Position, &InteractionRadius, &DynamicEnabled),
        With<DynamicObject>,
    >,
    mut _trigger_events: MessageWriter<DynamicTriggerEvent>,
) {
    for (_player_entity, player_pos) in character_query.iter() {
        for (_dynamic_entity, dynamic_pos, radius, enabled) in dynamic_query.iter() {
            if !enabled.0 {
                continue;
            }

            let _distance = player_pos.0.distance(dynamic_pos.0);
            if _distance <= radius.0 {
                // TODO: Check if player pressed Interact action (requires CharacterAction from game-networking)
                // When Interact is pressed within radius, emit:
                // trigger_events.write(DynamicTriggerEvent { object, trigger_type: TriggerType::PlayerOnInteract, source: player });
            }
        }
    }
}

/// Detect onEntitySpawn triggers — fires once when a dynamic object is first created.
pub fn detect_spawn_triggers(
    query: Query<(Entity, &DynamicEnabled), Added<DynamicObject>>,
    mut trigger_events: MessageWriter<DynamicTriggerEvent>,
) {
    for (entity, enabled) in query.iter() {
        if !enabled.0 {
            continue;
        }

        trigger_events.write(DynamicTriggerEvent {
            object: entity,
            trigger_type: TriggerType::OnEntitySpawn,
            source: entity, // Self-triggered
        });
    }
}

/// Read DynamicTriggerEvents, look up matching triggers in DynamicBehavior,
/// and emit DynamicActionEvents for each action in the trigger's action list.
pub fn dispatch_trigger_actions(
    mut trigger_events: MessageReader<DynamicTriggerEvent>,
    behavior_query: Query<&DynamicBehavior>,
    mut action_events: MessageWriter<DynamicActionEvent>,
) {
    for event in trigger_events.read() {
        let Ok(behavior) = behavior_query.get(event.object) else {
            continue;
        };

        for trigger_def in &behavior.triggers {
            if trigger_def.trigger_type == event.trigger_type {
                for action_def in &trigger_def.actions {
                    action_events.write(DynamicActionEvent {
                        object: event.object,
                        action: action_def.clone(),
                        source: event.source,
                    });
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Resolve a collision event into (player, dynamic_object) pair.
fn resolve_collision_pair(
    collider1: Entity,
    collider2: Entity,
    body1: Option<Entity>,
    body2: Option<Entity>,
    character_query: &Query<Entity, With<CharacterMarker>>,
    dynamic_query: &Query<Entity, (With<DynamicObject>, With<DynamicEnabled>)>,
) -> Option<(Entity, Entity)> {
    let mut player = None;
    let mut dynamic = None;

    for &entity in &[collider1, collider2] {
        if character_query.contains(entity) {
            player = Some(entity);
        }
        if dynamic_query.contains(entity) {
            dynamic = Some(entity);
        }
    }

    for body in [body1, body2].into_iter().flatten() {
        if player.is_none() && character_query.contains(body) {
            player = Some(body);
        }
        if dynamic.is_none() && dynamic_query.contains(body) {
            dynamic = Some(body);
        }
    }

    match (player, dynamic) {
        (Some(p), Some(d)) => Some((p, d)),
        _ => None,
    }
}

fn is_enabled(query: &Query<&DynamicEnabled>, entity: Entity) -> bool {
    query.get(entity).is_ok_and(|e| e.0)
}
