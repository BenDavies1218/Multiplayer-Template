//! Visual interpolation sub-plugin — smooth rendering between network ticks.
//!
//! Extracted from [`FirstPersonPlugin`](crate::renderer::FirstPersonPlugin) so
//! it can be added independently for debugging or selective composition.

use avian3d::prelude::*;
use bevy::prelude::*;
use game_protocol::FloorMarker;
use game_protocol::ProjectileMarker;
use lightyear::prelude::*;
use lightyear_frame_interpolation::{FrameInterpolate, FrameInterpolationPlugin};

/// Adds frame interpolation for `Position` and `Rotation` on predicted
/// entities, and disables rollback on projectiles.
pub struct VisualInterpolationPlugin;

impl Plugin for VisualInterpolationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameInterpolationPlugin::<Position>::default());
        app.add_plugins(FrameInterpolationPlugin::<Rotation>::default());
        app.add_observer(add_visual_interpolation_components);
        app.add_systems(Last, disable_projectile_rollback);
    }
}

fn add_visual_interpolation_components(
    trigger: On<Add, Position>,
    query: Query<Entity, (With<Predicted>, Without<FloorMarker>)>,
    mut commands: Commands,
) {
    if !query.contains(trigger.entity) {
        return;
    }
    commands.entity(trigger.entity).insert((
        FrameInterpolate::<Position> {
            trigger_change_detection: true,
            ..default()
        },
        FrameInterpolate::<Rotation> {
            trigger_change_detection: true,
            ..default()
        },
    ));
}

#[allow(clippy::type_complexity)]
fn disable_projectile_rollback(
    mut commands: Commands,
    projectile_query: Query<
        Entity,
        (
            With<Predicted>,
            With<ProjectileMarker>,
            Without<DisableRollback>,
        ),
    >,
) {
    for entity in &projectile_query {
        commands.entity(entity).insert(DisableRollback);
    }
}
