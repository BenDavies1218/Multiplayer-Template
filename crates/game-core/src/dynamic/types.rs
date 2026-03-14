use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Entity type
// ---------------------------------------------------------------------------

/// Classifies what kind of scene entity a dynamic object wraps.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum EntityType {
    Mesh,
    Light,
    #[default]
    Empty,
    Camera,
}

// ---------------------------------------------------------------------------
// Trigger types
// ---------------------------------------------------------------------------

/// The kind of event that activates a dynamic object's action list.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TriggerType {
    /// Player presses Interact within radius.
    PlayerOnInteract,
    /// Player enters the object's sensor collider.
    PlayerOnEnter,
    /// Player exits the object's sensor collider.
    PlayerOnExit,
    /// A projectile hits the object.
    PlayerOnShoot,
    /// The object's health drops below a threshold.
    OnEntityHealth,
    /// Fires once when the entity first spawns (startup animations, lights).
    OnEntitySpawn,
    /// Fires when the entity's own state transitions.
    OnStateChange,
    /// Fires on a repeating timer.
    OnTimer,
    /// Fires once after a delay from spawn.
    OnDelay,
    /// Fires when another entity's state changes.
    OnTargetStateChange,
}

// ---------------------------------------------------------------------------
// Action types
// ---------------------------------------------------------------------------

/// A discrete effect that fires when a trigger activates.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    // Universal
    ToggleState,
    SetState,
    Enable,
    Disable,
    Collect,
    DisplayText,
    HideText,
    PlaySound,
    Delay,
    SetVisibility,
    // Light effects
    StartLightEffect,
    StopLightEffect,
    // Mesh transforms
    MoveTo,
    RotateTo,
    ScaleTo,
    SetMaterialColor,
    // Camera (future)
    ActivateCamera,
    DeactivateCamera,
    // Legacy (keep for backward compat)
    PlayAnimation,
    StopAnimation,
    SetLightIntensity,
    SetLightColor,
}

// ---------------------------------------------------------------------------
// Definitions (deserialized from GltfExtras)
// ---------------------------------------------------------------------------

/// One action entry inside a trigger, e.g. `{ "type": "play_animation", "animation": "door_open" }`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionDef {
    #[serde(rename = "type")]
    pub action_type: ActionType,
    /// All remaining key-value pairs from the JSON object.
    #[serde(flatten)]
    pub params: HashMap<String, serde_json::Value>,
}

/// One trigger entry, e.g. `{ "type": "playerOnInteract", "radius": 2.0, "actions": [...] }`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerDef {
    #[serde(rename = "type")]
    pub trigger_type: TriggerType,
    /// Trigger-specific parameters (radius, threshold, delay, etc.).
    #[serde(flatten)]
    pub params: HashMap<String, serde_json::Value>,
    /// Ordered list of actions to execute when this trigger fires.
    pub actions: Vec<ActionDef>,
}

/// Optional initial state configuration for a dynamic object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicStateConfig {
    #[serde(default = "default_initial_state")]
    pub initial: String,
    #[serde(default)]
    pub toggle: bool,
}

fn default_initial_state() -> String {
    "idle".to_string()
}

// ---------------------------------------------------------------------------
// Config (loaded from JSON)
// ---------------------------------------------------------------------------

/// Light metadata attached to a dynamic node when `node_type` is `light`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightInfo {
    pub light_type: String,
    pub color: [f32; 3],
    pub intensity: f32,
}

/// Per-node config entry in `dynamic_objects_config.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicNodeConfig {
    #[serde(rename = "type", default)]
    pub node_type: EntityType,
    #[serde(default)]
    pub light_info: Option<LightInfo>,
    #[serde(default)]
    pub triggers: Vec<TriggerDef>,
    #[serde(default)]
    pub state: Option<DynamicStateConfig>,
}

/// Root config for dynamic objects. Loaded from `dynamic_objects_config.json`.
#[derive(Resource, Debug, Clone, Serialize, Deserialize, Default)]
pub struct DynamicObjectsConfig {
    #[serde(default)]
    pub nodes: HashMap<String, DynamicNodeConfig>,
}

// ---------------------------------------------------------------------------
// ECS Components
// ---------------------------------------------------------------------------

/// Marker + metadata for a dynamic object entity. Replicated to clients.
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DynamicObject {
    /// The `dynamic_type` from Blender custom properties (e.g. "door", "pickup", "light").
    pub object_type: String,
    /// Unique identifier derived from the Blender node name.
    pub object_id: String,
    /// What kind of scene entity this wraps (mesh, light, empty, camera).
    #[serde(default)]
    pub entity_type: EntityType,
}

/// Replicated state of a dynamic object (e.g. "open" / "closed").
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DynamicState {
    pub current: String,
    pub togglable: bool,
}

/// Holds the parsed trigger/action definitions. Server + viewer only, NOT replicated.
#[derive(Component, Debug, Clone)]
pub struct DynamicBehavior {
    pub triggers: Vec<TriggerDef>,
}

/// Interaction proximity radius for `PlayerOnInteract` triggers.
#[derive(Component, Debug, Clone)]
pub struct InteractionRadius(pub f32);

/// Whether this dynamic object is currently enabled.
/// Disabled objects ignore all triggers.
#[derive(Component, Debug, Clone)]
pub struct DynamicEnabled(pub bool);

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

/// Maps dynamic object IDs (Blender node names) to their ECS entities.
/// Used for cross-object `target` references in actions.
#[derive(Resource, Debug, Default)]
pub struct DynamicObjectRegistry {
    pub by_id: HashMap<String, Entity>,
}

impl DynamicObjectRegistry {
    pub fn get(&self, id: &str) -> Option<Entity> {
        self.by_id.get(id).copied()
    }
}
