use avian3d::prelude::*;
use bevy::prelude::*;

use super::events::*;
use super::spawn_points::SpawnPoints;
use super::zones::*;
use crate::character::CharacterMarker;

/// Detect when characters enter zones (server-only, runs in FixedUpdate)
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn detect_zone_collisions(
    mut collision_events: MessageReader<CollisionStart>,
    mut spawn_points: Option<ResMut<SpawnPoints>>,
    character_query: Query<Entity, With<CharacterMarker>>,
    death_zone_query: Query<(Entity, &Name, &Transform, &ZoneProperties), With<DeathZone>>,
    damage_zone_query: Query<(Entity, &Name, &DamageZone, &Transform, &ZoneProperties)>,
    trigger_zone_query: Query<(Entity, &Name, &TriggerZone, &Transform, &ZoneProperties)>,
    mut positions: Query<&mut Position, With<CharacterMarker>>,
    mut velocities: Query<&mut LinearVelocity, With<CharacterMarker>>,
    mut zone_entered_events: MessageWriter<ZoneEnteredEvent>,
) {
    for event in collision_events.read() {
        let mut player_entity = None;
        let mut zone_entity = None;

        let is_zone = |entity: Entity| -> bool {
            death_zone_query.contains(entity)
                || damage_zone_query.contains(entity)
                || trigger_zone_query.contains(entity)
        };

        // Check collider entities
        for &entity in &[event.collider1, event.collider2] {
            if character_query.contains(entity) {
                player_entity = Some(entity);
            }
            if is_zone(entity) {
                zone_entity = Some(entity);
            }
        }

        // Also check rigid body entities for both player and zone
        for body in [event.body1, event.body2].into_iter().flatten() {
            if player_entity.is_none() && character_query.contains(body) {
                player_entity = Some(body);
            }
            if zone_entity.is_none() && is_zone(body) {
                zone_entity = Some(body);
            }
        }

        let (Some(player), Some(zone)) = (player_entity, zone_entity) else {
            continue;
        };

        let player_position = positions.get(player).map(|p| p.0).unwrap_or_default();

        // Death zone: teleport player to spawn point
        if let Ok((_z, name, transform, props)) = death_zone_query.get(zone) {
            info!("Player {:?} entered death zone '{}'", player, name);

            if let Some(ref mut sp) = spawn_points {
                let spawn_pos = sp.next();
                if let Ok(mut pos) = positions.get_mut(player) {
                    pos.0 = spawn_pos;
                }
                if let Ok(mut vel) = velocities.get_mut(player) {
                    vel.0 = Vec3::ZERO;
                }
                info!("Respawned player at {:?}", spawn_pos);
            }

            zone_entered_events.write(ZoneEnteredEvent {
                player,
                player_position,
                zone,
                zone_name: name.to_string(),
                zone_type: ZoneType::DeathZone,
                zone_transform: *transform,
                properties: props.0.clone(),
            });
            continue;
        }

        // Damage zone: log for now (future: health system)
        if let Ok((_z, name, damage_zone, transform, props)) = damage_zone_query.get(zone) {
            info!(
                "Player {:?} entered damage zone '{}' (damage={}, interval={})",
                player, name, damage_zone.damage, damage_zone.interval
            );

            zone_entered_events.write(ZoneEnteredEvent {
                player,
                player_position,
                zone,
                zone_name: name.to_string(),
                zone_type: ZoneType::DamageZone,
                zone_transform: *transform,
                properties: props.0.clone(),
            });
            continue;
        }

        // Generic trigger zone: fire event
        if let Ok((_z, name, trigger_zone, transform, props)) = trigger_zone_query.get(zone) {
            info!(
                "Player {:?} entered trigger zone '{}' (event={})",
                player, name, trigger_zone.event_name
            );

            zone_entered_events.write(ZoneEnteredEvent {
                player,
                player_position,
                zone,
                zone_name: name.to_string(),
                zone_type: ZoneType::Trigger,
                zone_transform: *transform,
                properties: props.0.clone(),
            });
        }
    }
}

/// Detect when characters exit zones
#[allow(clippy::type_complexity)]
pub fn detect_zone_exits(
    mut collision_events: MessageReader<CollisionEnd>,
    character_query: Query<Entity, With<CharacterMarker>>,
    zone_query: Query<(
        Entity,
        &Name,
        &Transform,
        &ZoneProperties,
        Option<&DeathZone>,
        Option<&DamageZone>,
        Option<&TriggerZone>,
    )>,
    mut zone_exited_events: MessageWriter<ZoneExitedEvent>,
) {
    for event in collision_events.read() {
        let mut player_entity = None;
        let mut zone_entity = None;

        for &entity in &[event.collider1, event.collider2] {
            if character_query.contains(entity) {
                player_entity = Some(entity);
            }
            if zone_query.contains(entity) {
                zone_entity = Some(entity);
            }
        }

        if player_entity.is_none() {
            for body in [event.body1, event.body2].into_iter().flatten() {
                if character_query.contains(body) {
                    player_entity = Some(body);
                }
            }
        }

        let (Some(player), Some(zone)) = (player_entity, zone_entity) else {
            continue;
        };

        if let Ok((_z, name, _transform, props, death, damage, trigger)) = zone_query.get(zone) {
            let zone_type = if death.is_some() {
                ZoneType::DeathZone
            } else if damage.is_some() {
                ZoneType::DamageZone
            } else if trigger.is_some() {
                ZoneType::Trigger
            } else {
                continue;
            };

            info!("Player {:?} exited zone '{}'", player, name);

            zone_exited_events.write(ZoneExitedEvent {
                player,
                zone,
                zone_name: name.to_string(),
                zone_type,
                properties: props.0.clone(),
            });
        }
    }
}
