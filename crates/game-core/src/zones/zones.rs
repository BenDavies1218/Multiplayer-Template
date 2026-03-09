use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Component, Debug)]
pub struct SpawnPoint {
    pub index: u32,
}

#[derive(Component, Debug)]
pub struct DeathZone;

#[derive(Component, Debug)]
pub struct DamageZone {
    pub damage: f32,
    pub interval: f32,
}

#[derive(Component, Debug)]
pub struct TriggerZone {
    pub event_name: String,
}

/// Stores parsed custom properties from Blender glTF extras
#[derive(Component, Debug, Clone)]
pub struct ZoneProperties(pub HashMap<String, serde_json::Value>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZoneType {
    SpawnPoint,
    DeathZone,
    DamageZone,
    Trigger,
}
