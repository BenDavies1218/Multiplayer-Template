//! Input sub-plugin — runtime input device detection and camera-to-character sync.
//!
//! Extracted from [`ClientPlugin`](crate::client::ClientPlugin) so it can be
//! added independently for debugging or selective composition.

use bevy::prelude::*;
use lightyear::prelude::client::input::InputSystems;
use lightyear::prelude::is_in_rollback;

use crate::client_config::ActiveInputDevice;
use crate::input_device::{detect_input_device, rebuild_character_input_map};
use crate::movement::sync_camera_to_character;

/// Handles runtime input device detection, InputMap rebuilding, and
/// camera-yaw-to-ActionState synchronisation.
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        // Insert the active device resource (starts as KeyboardMouse, updated at runtime).
        app.init_resource::<ActiveInputDevice>();

        // Update: detect gamepad connect/disconnect, then rebuild InputMap if needed.
        app.add_systems(
            Update,
            (detect_input_device, rebuild_character_input_map).chain(),
        );

        // ── FixedPreUpdate: write camera yaw into ActionState BEFORE
        //    Lightyear captures it in BufferClientInputs. This ensures
        //    the server receives the same yaw the client uses for movement.
        app.add_systems(
            FixedPreUpdate,
            sync_camera_to_character
                .run_if(not(is_in_rollback))
                .after(InputSystems::WriteClientInputs)
                .before(InputSystems::BufferClientInputs),
        );
    }
}
