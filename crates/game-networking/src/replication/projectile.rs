//! Projectile replication
//!
//! Spawning and lifecycle systems for replicated projectile entities.

use avian3d::prelude::*;
use bevy::math::EulerRot;
use bevy::prelude::*;
use core::time::Duration;
use leafwing_input_manager::prelude::ActionState;
use lightyear::prelude::*;

use super::DespawnAfter;
use crate::protocol::{CameraOrientation, CharacterAction, CharacterMarker, ProjectileMarker};
use game_core::core_config::GameCoreConfig;

pub fn shoot_bullet(
    mut commands: Commands,
    query: Query<
        (
            &ActionState<CharacterAction>,
            &Position,
            &CameraOrientation,
            Option<&ControlledBy>,
        ),
        (
            // With<Predicted> matches client-side predicted entities.
            // With<Replicate> matches server-side authoritative entities (Replicate is only present on server).
            Or<(With<Predicted>, With<Replicate>)>,
            With<CharacterMarker>,
        ),
    >,
    config: Res<GameCoreConfig>,
    time: Res<Time<Fixed>>,
) {
    for (action_state, position, orientation, controlled_by) in &query {
        if !action_state.just_pressed(&CharacterAction::Fire) {
            continue;
        }

        let direction =
            Quat::from_euler(EulerRot::YXZ, orientation.yaw, orientation.pitch, 0.0) * Vec3::NEG_Z;
        let velocity = direction * config.projectile.velocity;

        let bullet_bundle = (
            Name::new("Projectile"),
            ProjectileMarker,
            RigidBody::Dynamic,
            *position,
            Rotation::default(),
            LinearVelocity(velocity),
        );

        if let Some(controlled_by) = controlled_by {
            // Server side — replicate to clients with replicate-once physics overrides
            let mut position_override = ComponentReplicationOverrides::<Position>::default();
            position_override.global_override(ComponentReplicationOverride {
                replicate_once: true,
                ..default()
            });
            let mut rotation_override = ComponentReplicationOverrides::<Rotation>::default();
            rotation_override.global_override(ComponentReplicationOverride {
                replicate_once: true,
                ..default()
            });
            let mut lv_override = ComponentReplicationOverrides::<LinearVelocity>::default();
            lv_override.global_override(ComponentReplicationOverride {
                replicate_once: true,
                ..default()
            });
            let mut av_override = ComponentReplicationOverrides::<AngularVelocity>::default();
            av_override.global_override(ComponentReplicationOverride {
                replicate_once: true,
                ..default()
            });

            commands.spawn((
                bullet_bundle,
                PreSpawned::default(),
                Replicate::to_clients(NetworkTarget::All),
                PredictionTarget::to_clients(NetworkTarget::All),
                ControlledBy {
                    owner: controlled_by.owner,
                    lifetime: Default::default(),
                },
                DespawnAfter {
                    spawned_at: time.elapsed_secs(),
                    lifetime: Duration::from_millis(config.projectile.lifetime_ms),
                },
                position_override,
                rotation_override,
                lv_override,
                av_override,
            ));
        } else {
            // Client side — pre-spawned local bullet; DespawnAfter added locally
            // since DespawnAfter is not replicated. Lightyear will match this entity
            // with the server's replicated bullet and won't remove local-only components.
            commands.spawn((
                bullet_bundle,
                PreSpawned::default(),
                DespawnAfter {
                    spawned_at: time.elapsed_secs(),
                    lifetime: Duration::from_millis(config.projectile.lifetime_ms),
                },
            ));
        }
    }
}
