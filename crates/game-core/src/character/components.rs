use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CharacterMarker;

/// Identifies which visual model a character uses.
/// Replicated from server to all clients.
/// The value is a key into the client's model catalog (e.g. "default", "soldier").
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CharacterModelId(pub String);

impl Default for CharacterModelId {
    fn default() -> Self {
        Self("default".to_string())
    }
}

/// Marker for hitbox child entities attached to a character.
#[derive(Component, Debug)]
pub struct CharacterHitboxMarker;

/// Describes a hitbox region (e.g. head, torso, limb).
/// Attached to child collider entities of the character.
/// `base_damage` is read from the glTF extras set by the modeller in Blender.
#[derive(Component, Debug, Clone)]
pub struct HitboxRegion {
    pub name: String,
    pub base_damage: f32,
}
