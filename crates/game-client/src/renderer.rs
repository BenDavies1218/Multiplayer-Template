//! First-person renderer plugin — convenience wrapper that assembles all
//! rendering sub-plugins plus the camera-follow glue system.
//!
//! Adds [`CameraPlugin`], [`CharacterRenderingPlugin`], [`ClientSkyboxPlugin`],
//! [`VisualInterpolationPlugin`], [`ProjectileCosmeticsPlugin`],
//! [`CursorPlugin`], and [`AnimationPlugin`].
//! Apps can also add these sub-plugins individually for debugging.

use bevy::prelude::*;
use game_camera::{CameraConfig, CameraPlugin, GameCamera};
use game_core::GameSimulationConfig;
use game_networking::protocol::{CharacterMarker, CrouchState};
use lightyear::prelude::*;

use crate::animation_plugin::AnimationPlugin;
use crate::character_rendering::CharacterRenderingPlugin;
use crate::client_config::GameClientConfig;
use crate::client_skybox_plugin::ClientSkyboxPlugin;
use crate::cursor_plugin::CursorPlugin;
use crate::projectile_cosmetics_plugin::ProjectileCosmeticsPlugin;
use crate::visual_interpolation_plugin::VisualInterpolationPlugin;

pub struct FirstPersonPlugin {
    pub camera_config: CameraConfig,
}

impl Default for FirstPersonPlugin {
    fn default() -> Self {
        Self {
            camera_config: CameraConfig::first_person(),
        }
    }
}

impl Plugin for FirstPersonPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CameraPlugin {
            config: self.camera_config.clone(),
        });
        app.add_plugins(CharacterRenderingPlugin);
        app.add_plugins(ClientSkyboxPlugin);
        app.add_plugins(VisualInterpolationPlugin);
        app.add_plugins(ProjectileCosmeticsPlugin);
        app.add_plugins(CursorPlugin);
        app.add_plugins(AnimationPlugin);

        app.add_systems(Update, fps_camera_follow);
    }
}

#[allow(clippy::type_complexity)]
fn fps_camera_follow(
    mut camera_query: Query<&mut Transform, (With<GameCamera>, Without<CharacterMarker>)>,
    player_query: Query<
        (&Transform, &CrouchState),
        (With<CharacterMarker>, With<Predicted>, Without<GameCamera>),
    >,
    sim_config: Res<GameSimulationConfig>,
    client_config: Res<GameClientConfig>,
) {
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    if let Some((player_transform, crouch_state)) = player_query.iter().next() {
        let capsule_height = if crouch_state.0 {
            sim_config.movement.crouch_capsule_height
        } else {
            sim_config.character.capsule_height
        };
        let eye_height = capsule_height / 2.0
            + sim_config.character.capsule_radius
            + client_config.rendering.eye_height_offset;
        camera_transform.translation =
            player_transform.translation + Vec3::new(0.0, eye_height, 0.0);
    }
}
