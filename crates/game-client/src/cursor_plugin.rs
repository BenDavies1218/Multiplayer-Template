//! Cursor sub-plugin — cursor grab/release handling.
//!
//! Extracted from [`FirstPersonPlugin`](crate::renderer::FirstPersonPlugin) so
//! it can be added independently for debugging or selective composition.

use bevy::{
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};
use game_core::core_config::parse_key_code;

use crate::client_config::{parse_mouse_button, GameClientConfig};

/// Manages cursor grab/release based on configured key/button bindings.
pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, setup_cursor_grab);
    }
}

fn setup_cursor_grab(
    mut cursor_options: Single<&mut CursorOptions>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    key: Res<ButtonInput<KeyCode>>,
    config: Res<GameClientConfig>,
) {
    let release_key = parse_key_code(&config.input.cursor_release_key).unwrap_or(KeyCode::Escape);
    let grab_button =
        parse_mouse_button(&config.input.cursor_grab_button).unwrap_or(MouseButton::Left);
    if key.just_pressed(release_key) {
        cursor_options.visible = true;
        cursor_options.grab_mode = CursorGrabMode::None;
    }
    if mouse_button.just_pressed(grab_button) {
        cursor_options.visible = false;
        cursor_options.grab_mode = CursorGrabMode::Locked;
    }
}
