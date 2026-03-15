//! Periodic diagnostics logging for server mode.
//!
//! Logs performance metrics at a configurable interval using `info!()`.

use bevy::diagnostic::{
    DiagnosticsStore, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
};
use bevy::prelude::*;
use lightyear_prediction::diagnostics::PredictionDiagnosticsPlugin;

/// Timer resource controlling log output frequency.
#[derive(Resource)]
struct DiagnosticsLogTimer(Timer);

pub(crate) fn setup_server_log(app: &mut App, interval_secs: f64) {
    app.insert_resource(DiagnosticsLogTimer(Timer::from_seconds(
        interval_secs as f32,
        TimerMode::Repeating,
    )));
    app.add_systems(Update, log_diagnostics);
}

fn log_diagnostics(
    time: Res<Time>,
    mut timer: ResMut<DiagnosticsLogTimer>,
    diagnostics: Res<DiagnosticsStore>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    let frame_time = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    let entities = diagnostics
        .get(&EntityCountDiagnosticsPlugin::ENTITY_COUNT)
        .and_then(|d| d.value())
        .unwrap_or(0.0) as u64;

    let rollbacks = diagnostics
        .get(&PredictionDiagnosticsPlugin::ROLLBACKS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    let depth = diagnostics
        .get(&PredictionDiagnosticsPlugin::ROLLBACK_DEPTH)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    info!(
        "[DIAGNOSTICS] FPS: {:.1} | Frame: {:.2}ms | Entities: {} | Rollbacks: {:.1}/s | Depth: {:.1}",
        fps, frame_time, entities, rollbacks, depth
    );
}
