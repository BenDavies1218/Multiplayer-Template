//! Lifecycle sub-plugin — handles new predicted character entities.
//!
//! Extracted from [`ClientPlugin`](crate::client::ClientPlugin) so it can be
//! added independently for debugging or selective composition.

use bevy::prelude::*;

use crate::character::handle_new_character;

/// Attaches `InputMap`, physics bundle, and orientation state to newly
/// spawned predicted characters.
pub struct LifecyclePlugin;

impl Plugin for LifecyclePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_new_character);
    }
}
