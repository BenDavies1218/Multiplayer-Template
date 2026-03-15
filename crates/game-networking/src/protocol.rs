use avian3d::prelude::*;
use bevy::prelude::*;
use lightyear::input::prelude::InputConfig;
use lightyear::prelude::input::leafwing;
use lightyear::prelude::*;

use super::rollback::{
    angular_velocity_should_rollback, linear_velocity_should_rollback, position_should_rollback,
    rotation_should_rollback,
};
use game_core::character::CharacterModelId;
use game_core::dynamic::{DynamicObject, DynamicState};

// Re-export so callers that previously used `protocol::set_prediction_speed` still compile.
pub use super::rollback::{init_rollback_config, set_prediction_speed};

// Re-export CharacterMarker from game-core so it's accessible via `game_networking::protocol::CharacterMarker`
pub use game_core::character::CharacterMarker;

// Re-export all shared type definitions from game-protocol
pub use game_protocol::*;

// Protocol
#[derive(Clone)]
pub struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        // Add input manager plugin for CharacterAction, with rebroadcasting enabled so that inputs are sent to the server for processing.
        app.add_plugins(leafwing::InputPlugin::<CharacterAction> {
            config: InputConfig::<CharacterAction> {
                rebroadcast_inputs: true,
                ..default()
            },
        });

        app.register_component::<ColorComponent>();

        app.register_component::<Name>();

        app.register_component::<CharacterMarker>();

        app.register_component::<ProjectileMarker>();

        app.register_component::<FloorMarker>();

        // Camera orientation - NOT predicted, client authority
        // Client updates this and server reads it directly
        app.register_component::<CameraOrientation>();

        app.register_component::<CharacterModelId>();

        app.register_component::<CrouchState>().add_prediction();

        // Dynamic object replication (server -> client)
        app.register_component::<DynamicObject>();
        app.register_component::<DynamicState>();

        // Fully replicated, but not visual, so no need for lerp/corrections:
        app.register_component::<LinearVelocity>()
            .add_prediction()
            .add_should_rollback(linear_velocity_should_rollback);

        app.register_component::<AngularVelocity>()
            .add_prediction()
            .add_should_rollback(angular_velocity_should_rollback);

        // Position and Rotation use `correction_fn` to smear rollback errors
        // over a few frames visually, and `interpolation_fn` to smooth
        // rendering between FixedUpdate ticks.
        app.register_component::<Position>()
            .add_prediction()
            .add_should_rollback(position_should_rollback)
            .add_linear_correction_fn()
            .add_linear_interpolation();

        app.register_component::<Rotation>()
            .add_prediction()
            .add_should_rollback(rotation_should_rollback)
            .add_linear_correction_fn()
            .add_linear_interpolation();
    }
}
