use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};

use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use lightyear::input::prelude::InputConfig;
use lightyear::prelude::input::leafwing;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

use crate::character::CharacterModelId;
use crate::core_config::RollbackConfig;

static ROLLBACK_CONFIG: OnceLock<RollbackConfig> = OnceLock::new();

/// Initialize the global rollback thresholds from a `RollbackConfig`.
/// Should be called once during plugin setup.
pub fn init_rollback_config(config: RollbackConfig) {
    ROLLBACK_CONFIG.set(config).ok();
}

fn rollback_thresholds() -> &'static RollbackConfig {
    ROLLBACK_CONFIG.get_or_init(RollbackConfig::default)
}

/// Current predicted entity horizontal speed, updated each FixedUpdate tick by the client.
/// Used to scale the position rollback threshold dynamically — the legitimate prediction
/// offset grows with speed, so the threshold must too.
static CURRENT_SPEED: AtomicU32 = AtomicU32::new(0);

/// Call this from client FixedUpdate with the controlled character's horizontal speed.
pub fn set_prediction_speed(speed: f32) {
    CURRENT_SPEED.store(speed.to_bits(), Ordering::Relaxed);
}

fn prediction_speed() -> f32 {
    f32::from_bits(CURRENT_SPEED.load(Ordering::Relaxed))
}

// Components

#[derive(Component, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct ColorComponent(pub Color);

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CharacterMarker;

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FloorMarker;

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ProjectileMarker;

#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BlockMarker;

/// Camera orientation component
/// Client updates this locally, server reads from ActionState metadata via replication
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CameraOrientation {
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Reflect, Serialize, Deserialize)]
pub enum CharacterAction {
    Move,
    Jump,
    Sprint,
    Crouch,
    Shoot,
    Look, // Camera yaw/pitch as DualAxis
}

impl Actionlike for CharacterAction {
    fn input_control_kind(&self) -> InputControlKind {
        match self {
            Self::Move => InputControlKind::DualAxis,
            Self::Look => InputControlKind::DualAxis,
            Self::Jump => InputControlKind::Button,
            Self::Sprint => InputControlKind::Button,
            Self::Crouch => InputControlKind::Button,
            Self::Shoot => InputControlKind::Button,
        }
    }
}

/// Tracks whether a character is currently crouching.
/// Replicated for collider sync between client and server.
#[derive(Component, Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
pub struct CrouchState(pub bool);

// Protocol
#[derive(Clone)] // Added Clone
pub(crate) struct ProtocolPlugin;

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        // Leafwing input for WASD/Jump/Shoot
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

        app.register_component::<BlockMarker>();

        // Camera orientation - NOT predicted, client authority
        // Client updates this and server reads it directly
        app.register_component::<CameraOrientation>();

        app.register_component::<CharacterModelId>();

        app.register_component::<CrouchState>().add_prediction();

        // Fully replicated, but not visual, so no need for lerp/corrections:
        app.register_component::<LinearVelocity>()
            .add_prediction()
            .add_should_rollback(linear_velocity_should_rollback);

        app.register_component::<AngularVelocity>()
            .add_prediction()
            .add_should_rollback(angular_velocity_should_rollback);

        // app.register_component::<ComputedMass>().add_prediction();

        // Position and Rotation have a `correction_fn` set, which is used to smear rollback errors
        // over a few frames, just for the rendering part in postudpate.
        //
        // They also set `interpolation_fn` which is used by the VisualInterpolationPlugin to smooth
        // out rendering between fixedupdate ticks.
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

fn position_should_rollback(this: &Position, that: &Position) -> bool {
    // Compare only horizontal (XZ) distance — Y-position drift from unreplicated Avian
    // ground contact caches would trigger constant false positives.
    //
    // The threshold is dynamic: `base + speed × lag_budget`.
    // Rationale: the client predicts N ticks ahead of the server's last confirmed state.
    // At full speed the legitimate prediction offset is `speed × N × dt`, which can be
    // up to ~0.12m on a low-latency connection. A fixed small threshold triggers rollback
    // on every server ack even when client and server are on identical trajectories.
    let diff = this.0 - that.0;
    let horiz_dist = Vec2::new(diff.x, diff.z).length();
    let cfg = rollback_thresholds();
    let threshold = cfg.position + prediction_speed() * cfg.position_speed_factor;
    if horiz_dist >= threshold {
        warn!(
            "[position-rollback] horiz_dist={horiz_dist:.4}m >= threshold={threshold:.4}m \
             predicted=({:.3},{:.3},{:.3}) server=({:.3},{:.3},{:.3})",
            this.x, this.y, this.z, that.x, that.y, that.z
        );
    }
    horiz_dist >= threshold
}

fn rotation_should_rollback(this: &Rotation, that: &Rotation) -> bool {
    this.angle_between(*that) >= rollback_thresholds().rotation
}

fn linear_velocity_should_rollback(this: &LinearVelocity, that: &LinearVelocity) -> bool {
    (this.0 - that.0).length() >= rollback_thresholds().linear_velocity
}

fn angular_velocity_should_rollback(this: &AngularVelocity, that: &AngularVelocity) -> bool {
    (this.0 - that.0).length() >= rollback_thresholds().angular_velocity
}
