//! Projectile cosmetics sub-plugin — attaches visual meshes to projectiles.
//!
//! Extracted from [`FirstPersonPlugin`](crate::renderer::FirstPersonPlugin) so
//! it can be added independently for debugging or selective composition.

use avian3d::prelude::*;
use bevy::{color::palettes::css::MAGENTA, prelude::*};
use game_core::GameCoreConfig;
use game_protocol::ProjectileMarker;
use lightyear::prelude::*;

use crate::client_config::GameClientConfig;

/// Adds sphere meshes and physics colliders to newly predicted/replicated
/// projectile entities.
pub struct ProjectileCosmeticsPlugin;

impl Plugin for ProjectileCosmeticsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            add_projectile_cosmetics.before(RollbackSystems::Check),
        );
    }
}

#[allow(clippy::type_complexity)]
fn add_projectile_cosmetics(
    mut commands: Commands,
    projectile_query: Query<
        Entity,
        (
            Or<(Added<Predicted>, Added<Replicate>)>,
            With<ProjectileMarker>,
        ),
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<GameClientConfig>,
    core_config: Res<GameCoreConfig>,
) {
    for entity in &projectile_query {
        commands.entity(entity).insert((
            Mesh3d(meshes.add(Sphere::new(config.rendering.projectile_radius))),
            MeshMaterial3d(materials.add(Color::from(MAGENTA))),
            RigidBody::Dynamic,
            Collider::sphere(core_config.projectile.radius),
        ));
    }
}
