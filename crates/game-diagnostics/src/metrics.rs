//! Registers Bevy's built-in diagnostics plugins and Lightyear diagnostics.

use bevy::diagnostic::{
    EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
    SystemInformationDiagnosticsPlugin,
};
use bevy::prelude::*;

use crate::DiagnosticsMode;

pub(crate) fn register_metric_plugins(app: &mut App, mode: DiagnosticsMode) {
    // Frame time + FPS — all modes
    app.add_plugins(FrameTimeDiagnosticsPlugin::default());

    // Entity count — all modes
    app.add_plugins(EntityCountDiagnosticsPlugin::default());

    // System info (CPU, memory) — client and viewer only (needs windowed app)
    if mode != DiagnosticsMode::Server {
        app.add_plugins(SystemInformationDiagnosticsPlugin);
    }

    // Lightyear networking diagnostics — client and server only
    if mode == DiagnosticsMode::Client || mode == DiagnosticsMode::Server {
        // PredictionDiagnosticsPlugin registers rollback metrics into Bevy Diagnostics
        app.add_plugins(
            lightyear_prediction::diagnostics::PredictionDiagnosticsPlugin::default(),
        );
        // PingDiagnosticsPlugin registers RTT and jitter metrics
        app.add_plugins(lightyear_sync::ping::diagnostics::PingDiagnosticsPlugin::default());
    }
}
