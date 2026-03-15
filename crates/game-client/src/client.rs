//! Client plugin — convenience wrapper that assembles all client-side sub-plugins.
//!
//! Adds [`InputPlugin`], [`PredictionPlugin`], and [`LifecyclePlugin`].
//! Apps can also add these sub-plugins individually for debugging.

use bevy::prelude::*;

use crate::input_plugin::InputPlugin;
use crate::lifecycle_plugin::LifecyclePlugin;
use crate::prediction_plugin::PredictionPlugin;

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputPlugin);
        app.add_plugins(PredictionPlugin);
        app.add_plugins(LifecyclePlugin);
    }
}
