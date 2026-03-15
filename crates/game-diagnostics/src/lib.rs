//! Game Diagnostics — unified performance monitoring for all apps.
//!
//! Exports a single [`DiagnosticsPlugin`] with `client()`, `server()`, and `viewer()`
//! constructors following the existing plugin pattern.

use bevy::prelude::*;

mod metrics;
mod overlay;
mod server_log;
mod thresholds;

/// The operating mode for the diagnostics plugin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagnosticsMode {
    /// Client: on-screen overlay toggled with F3, includes networking metrics.
    Client,
    /// Server: periodic log output, includes networking metrics, no rendering.
    Server,
    /// Viewer: on-screen overlay toggled with F3, no networking metrics.
    Viewer,
}

/// Unified performance monitoring plugin.
///
/// Use `DiagnosticsPlugin::client()`, `::server()`, or `::viewer()` to construct.
pub struct DiagnosticsPlugin {
    pub mode: DiagnosticsMode,
    pub server_log_interval_secs: f64,
}

impl DiagnosticsPlugin {
    pub fn client() -> Self {
        Self {
            mode: DiagnosticsMode::Client,
            server_log_interval_secs: 10.0,
        }
    }

    pub fn server() -> Self {
        Self {
            mode: DiagnosticsMode::Server,
            server_log_interval_secs: 10.0,
        }
    }

    pub fn server_with_interval(interval_secs: f64) -> Self {
        Self {
            mode: DiagnosticsMode::Server,
            server_log_interval_secs: interval_secs,
        }
    }

    pub fn viewer() -> Self {
        Self {
            mode: DiagnosticsMode::Viewer,
            server_log_interval_secs: 10.0,
        }
    }
}

impl Plugin for DiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        metrics::register_metric_plugins(app, self.mode);

        match self.mode {
            DiagnosticsMode::Client | DiagnosticsMode::Viewer => {
                overlay::setup_overlay(app, self.mode);
            }
            DiagnosticsMode::Server => {
                server_log::setup_server_log(app, self.server_log_interval_secs);
            }
        }
    }
}
