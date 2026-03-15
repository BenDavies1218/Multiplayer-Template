//! Game Protocol — Shared type definitions for the multiplayer protocol.
//!
//! This leaf crate contains the type definitions (components, actions, markers)
//! that are shared across the networking stack. By living in its own crate,
//! these types can be depended on by `game-core` without pulling in the full
//! `game-networking` crate and its heavier dependencies (Avian3d, full Lightyear
//! client/server features, etc.).
//!
//! **Note:** This crate only defines types. Lightyear registration and plugin
//! setup remain in `game-networking`.

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use serde::{Deserialize, Serialize};

// Components

#[derive(Component, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct ColorComponent(pub Color);

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FloorMarker;

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ProjectileMarker;

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BlockMarker;

/// Camera orientation component.
/// Client updates this locally, server reads from ActionState metadata via replication.
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CameraOrientation {
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Reflect, Serialize, Deserialize)]
pub enum CharacterAction {
    // Movement
    Move, // DualAxis
    Look, // DualAxis
    Jump,
    Sprint,
    Crouch,
    Prone,
    MountLedge,

    // Combat
    Fire,
    AimDownSights,
    Reload,
    PrimaryWeapon,
    SecondaryWeapon,
    Interact,
    LethalEquipment,
    TacticalEquipment,
    Melee,
    WeaponInspect,
    ArmorPlate,
    AlternateFire,

    // Killstreaks & field upgrades
    Killstreak1,
    Killstreak2,
    Killstreak3,
    FieldUpgrade,

    // Communication
    TextChat,
    TeamChat,
    Ping,
    PushToTalk,
    Gesture1,
    Gesture2,
    Gesture3,
    Gesture4,

    // Misc
    Scoreboard,
    Map,
    Inventory,
    Pause,
    NightVision,
}

impl Actionlike for CharacterAction {
    fn input_control_kind(&self) -> InputControlKind {
        match self {
            Self::Move | Self::Look => InputControlKind::DualAxis,
            _ => InputControlKind::Button,
        }
    }
}

/// Tracks whether a character is currently crouching.
/// Replicated for collider sync between client and server.
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct CrouchState(pub bool);
