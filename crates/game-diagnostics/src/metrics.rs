//! Registers Bevy's built-in diagnostics plugins and Lightyear diagnostics.

use bevy::diagnostic::{
    EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, SystemInformationDiagnosticsPlugin,
};
use bevy::prelude::*;

use crate::DiagnosticsMode;

pub(crate) fn register_metric_plugins(app: &mut App, mode: DiagnosticsMode) {
    // Frame time + FPS — all modes
    app.add_plugins(FrameTimeDiagnosticsPlugin::default());

    // Entity count — all modes
    app.add_plugins(EntityCountDiagnosticsPlugin::default());

    // System info (CPU, memory) — all modes
    app.add_plugins(SystemInformationDiagnosticsPlugin);

    // Lightyear networking diagnostics — client and server only
    // Guard against double-registration: Lightyear's ClientPlugins/ServerPlugins may already add these.
    if mode == DiagnosticsMode::Client || mode == DiagnosticsMode::Server {
        if !app.is_plugin_added::<lightyear_prediction::diagnostics::PredictionDiagnosticsPlugin>() {
            app.add_plugins(lightyear_prediction::diagnostics::PredictionDiagnosticsPlugin::default());
        }
        if !app.is_plugin_added::<lightyear_sync::ping::diagnostics::PingDiagnosticsPlugin>() {
            app.add_plugins(lightyear_sync::ping::diagnostics::PingDiagnosticsPlugin::default());
        }
    }
}
