//! Game Diagnostics — unified performance monitoring for all apps.
//!
//! Exports a single [`DiagnosticsPlugin`] with `client()`, `server()`, and `viewer()`
//! constructors following the existing plugin pattern.

use bevy::prelude::*;

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
}

impl DiagnosticsPlugin {
    pub fn client() -> Self {
        Self {
            mode: DiagnosticsMode::Client,
        }
    }

    pub fn server() -> Self {
        Self {
            mode: DiagnosticsMode::Server,
        }
    }

    pub fn viewer() -> Self {
        Self {
            mode: DiagnosticsMode::Viewer,
        }
    }
}

impl Plugin for DiagnosticsPlugin {
    fn build(&self, _app: &mut App) {
        // TODO: wire up sub-plugins per mode
    }
}
