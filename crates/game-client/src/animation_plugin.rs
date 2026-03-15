//! Animation sub-plugin — auto-plays glTF animations.
//!
//! Extracted from [`FirstPersonPlugin`](crate::renderer::FirstPersonPlugin) so
//! it can be added independently for debugging or selective composition.

use bevy::prelude::*;

/// Automatically plays all animations from loaded glTF scenes on loop.
pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, auto_play_gltf_animations);
    }
}

/// Auto-play all animations from loaded glTF scenes on loop.
fn auto_play_gltf_animations(
    mut players: Query<(&mut AnimationPlayer, &AnimationGraphHandle), Added<AnimationPlayer>>,
    graphs: Res<Assets<AnimationGraph>>,
) {
    for (mut player, graph_handle) in &mut players {
        let Some(graph) = graphs.get(&graph_handle.0) else {
            continue;
        };
        for index in graph.nodes() {
            player.play(index).repeat();
        }
    }
}
