//! On-screen diagnostics overlay for client and viewer modes.
//!
//! Toggled with P. Shows FPS, frame time, entity count, and (for Client mode)
//! networking metrics from Lightyear.

use bevy::diagnostic::{
    DiagnosticsStore, EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin,
    SystemInformationDiagnosticsPlugin,
};
use bevy::prelude::*;

use crate::DiagnosticsMode;

/// Marker component for the overlay root node.
#[derive(Component)]
struct DiagnosticsOverlay;

/// Marker component for the overlay text node.
#[derive(Component)]
struct DiagnosticsOverlayText;

/// Resource tracking which mode we're in, so the update system knows what to display.
#[derive(Resource, Clone, Copy)]
struct OverlayMode(DiagnosticsMode);

pub(crate) fn setup_overlay(app: &mut App, mode: DiagnosticsMode) {
    app.insert_resource(OverlayMode(mode));
    app.add_systems(Startup, spawn_overlay);
    app.add_systems(Update, (toggle_overlay, update_overlay));
}

fn spawn_overlay(mut commands: Commands) {
    commands
        .spawn((
            DiagnosticsOverlay,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            Visibility::Hidden,
        ))
        .with_child((
            DiagnosticsOverlayText,
            Text::new("Diagnostics"),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(0.0, 1.0, 0.0)),
        ));
}

fn toggle_overlay(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Visibility, With<DiagnosticsOverlay>>,
) {
    if keyboard.just_pressed(KeyCode::KeyP) {
        for mut visibility in query.iter_mut() {
            *visibility = match *visibility {
                Visibility::Hidden => Visibility::Visible,
                _ => Visibility::Hidden,
            };
        }
    }
}

fn update_overlay(
    diagnostics: Res<DiagnosticsStore>,
    mode: Res<OverlayMode>,
    overlay_query: Query<&Visibility, With<DiagnosticsOverlay>>,
    mut text_query: Query<&mut Text, With<DiagnosticsOverlayText>>,
) {
    // Only update text when visible
    let Ok(visibility) = overlay_query.single() else {
        return;
    };
    if *visibility == Visibility::Hidden {
        return;
    }

    let mut lines: Vec<String> = Vec::new();

    // FPS
    if let Some(fps) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
    {
        lines.push(format!("FPS: {:.1}", fps));
    }

    // Frame time
    if let Some(frame_time) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|d| d.smoothed())
    {
        lines.push(format!("Frame: {:.2}ms", frame_time));
    }

    // Entity count
    if let Some(entities) = diagnostics
        .get(&EntityCountDiagnosticsPlugin::ENTITY_COUNT)
        .and_then(|d| d.value())
    {
        lines.push(format!("Entities: {}", entities as u64));
    }

    // System info (CPU, memory) — client and viewer only
    if mode.0 != DiagnosticsMode::Server {
        if let Some(cpu) = diagnostics
            .get(&SystemInformationDiagnosticsPlugin::SYSTEM_CPU_USAGE)
            .and_then(|d| d.smoothed())
        {
            lines.push(format!("CPU: {:.1}%", cpu));
        }
        if let Some(mem) = diagnostics
            .get(&SystemInformationDiagnosticsPlugin::SYSTEM_MEM_USAGE)
            .and_then(|d| d.smoothed())
        {
            lines.push(format!("RAM: {:.0}MB", mem));
        }
    }

    // Networking metrics — client only
    if mode.0 == DiagnosticsMode::Client {
        use lightyear_sync::ping::diagnostics::PingDiagnosticsPlugin;

        if let Some(rtt) = diagnostics
            .get(&PingDiagnosticsPlugin::RTT)
            .and_then(|d| d.smoothed())
        {
            lines.push(format!("RTT: {:.1}ms", rtt));
        }
        if let Some(jitter) = diagnostics
            .get(&PingDiagnosticsPlugin::JITTER)
            .and_then(|d| d.smoothed())
        {
            lines.push(format!("Jitter: {:.1}ms", jitter));
        }

        use lightyear_prediction::diagnostics::PredictionDiagnosticsPlugin;

        if let Some(rollbacks) = diagnostics
            .get(&PredictionDiagnosticsPlugin::ROLLBACKS)
            .and_then(|d| d.smoothed())
        {
            lines.push(format!("Rollbacks: {:.1}/s", rollbacks));
        }
        if let Some(depth) = diagnostics
            .get(&PredictionDiagnosticsPlugin::ROLLBACK_DEPTH)
            .and_then(|d| d.smoothed())
        {
            lines.push(format!("Depth: {:.1}", depth));
        }
    }

    let text_content = lines.join("\n");

    // Update the text child — query for Text children of the overlay
    for mut text in text_query.iter_mut() {
        **text = text_content.clone();
    }
}
