use bevy::prelude::*;

use super::types::{ActionDef, TriggerType};

/// Fired by trigger detection systems when a dynamic object's trigger condition is met.
#[derive(Message, Debug)]
pub struct DynamicTriggerEvent {
    /// The dynamic object entity whose trigger fired.
    pub object: Entity,
    /// Which trigger type activated.
    pub trigger_type: TriggerType,
    /// The entity that caused the trigger (e.g. the player).
    pub source: Entity,
}

/// Fired by the action dispatcher for each individual action to execute.
#[derive(Message, Debug)]
pub struct DynamicActionEvent {
    /// The dynamic object entity being acted upon.
    pub object: Entity,
    /// The action definition with type and params.
    pub action: ActionDef,
    /// The entity that originally caused the trigger.
    pub source: Entity,
}
