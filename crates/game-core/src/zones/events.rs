use bevy::prelude::*;
use std::collections::HashMap;

use super::zones::ZoneType;

#[derive(Message, Debug)]
pub struct ZoneEnteredEvent {
    pub player: Entity,
    pub player_position: Vec3,
    pub zone: Entity,
    pub zone_name: String,
    pub zone_type: ZoneType,
    pub zone_transform: Transform,
    pub properties: HashMap<String, serde_json::Value>,
}

#[derive(Message, Debug)]
pub struct ZoneExitedEvent {
    pub player: Entity,
    pub zone: Entity,
    pub zone_name: String,
    pub zone_type: ZoneType,
    pub properties: HashMap<String, serde_json::Value>,
}
