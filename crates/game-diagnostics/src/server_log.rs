//! Periodic diagnostics logging for server mode.
//!
//! Logs performance metrics at a configurable interval using `info!()`.

use bevy::diagnostic::{
    DiagnosticsStore, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
    SystemInformationDiagnosticsPlugin,
};
use bevy::prelude::*;
use lightyear::connection::client::Connected;
use lightyear::prelude::server::ClientOf;
use lightyear_prediction::diagnostics::PredictionDiagnosticsPlugin;

use crate::thresholds;

/// Target tick rate from config, used to compare actual vs target.
#[derive(Resource)]
pub struct TargetTickRate(pub f64);

/// Counts FixedUpdate ticks between diagnostic log intervals.
#[derive(Resource, Default)]
struct TickCounter {
    count: u64,
    elapsed_secs: f64,
}

/// Timer resource controlling log output frequency.
#[derive(Resource)]
struct DiagnosticsLogTimer(Timer);

pub(crate) fn setup_server_log(app: &mut App, interval_secs: f64) {
    app.insert_resource(DiagnosticsLogTimer(Timer::from_seconds(
        interval_secs as f32,
        TimerMode::Repeating,
    )));
    app.init_resource::<TickCounter>();
    app.add_systems(FixedUpdate, count_fixed_ticks);
    app.add_systems(Update, log_diagnostics);
}

fn count_fixed_ticks(mut counter: ResMut<TickCounter>, time: Res<Time<Fixed>>) {
    counter.count += 1;
    counter.elapsed_secs += time.delta().as_secs_f64();
}

fn log_diagnostics(
    time: Res<Time>,
    mut timer: ResMut<DiagnosticsLogTimer>,
    diagnostics: Res<DiagnosticsStore>,
    clients: Query<Entity, (With<ClientOf>, With<Connected>)>,
    target_tick: Option<Res<TargetTickRate>>,
    mut tick_counter: ResMut<TickCounter>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }

    let client_count = clients.iter().count();

    let actual_tick_hz = if tick_counter.elapsed_secs > 0.0 {
        tick_counter.count as f64 / tick_counter.elapsed_secs
    } else {
        0.0
    };
    let target_hz = target_tick.as_ref().map(|t| t.0).unwrap_or(64.0);
    let tick_pct = if target_hz > 0.0 {
        (actual_tick_hz / target_hz) * 100.0
    } else {
        0.0
    };
    let tick_e = thresholds::tick_rate_pct(tick_pct).emoji();

    // Reset counter for next interval
    tick_counter.count = 0;
    tick_counter.elapsed_secs = 0.0;

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

    let cpu = diagnostics
        .get(&SystemInformationDiagnosticsPlugin::SYSTEM_CPU_USAGE)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    let mem = diagnostics
        .get(&SystemInformationDiagnosticsPlugin::SYSTEM_MEM_USAGE)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);

    let fps_e = thresholds::server_fps(fps).emoji();
    let frame_e = thresholds::frame_time_ms(frame_time).emoji();
    let rb_e = thresholds::rollbacks_per_sec(rollbacks).emoji();
    let dep_e = thresholds::rollback_depth(depth).emoji();

    info!(
        "[DIAGNOSTICS] Clients: {} | {} FPS: {:.1} | {} Frame: {:.2}ms | Entities: {} | CPU: {:.1}% | RAM: {:.0}MB | {} Rollbacks: {:.1}/s | {} Depth: {:.1} | {} Tick: {:.1}/{:.0} Hz ({:.0}%)",
        client_count, fps_e, fps, frame_e, frame_time, entities, cpu, mem, rb_e, rollbacks, dep_e, depth, tick_e, actual_tick_hz, target_hz, tick_pct
    );
}
