use bevy::prelude::*;
use crate::common::{GameScene, RunEntity};
use crate::config::HazardConfig;
use crate::message::messages::DamageMessage;
use crate::world::Survivor;

use super::state::{HazardMarker, HazardSpawnMessage, HazardZone, marker_transform};

const GROUND_Y: f32 = 0.0;

/// Advances every zone, applies cadence ticks to the survivor, and despawns expired zones.
pub fn tick_hazards(
    time: Res<Time>,
    cfg: Res<HazardConfig>,
    mut commands: Commands,
    mut damage_writer: MessageWriter<DamageMessage>,
    survivor: Option<Single<(Entity, &Transform), With<Survivor>>>,
    mut zones: Query<(Entity, &Transform, &mut HazardZone)>,
) {
    let dt = time.delta_secs();
    let interval = cfg.tick_interval;
    if !dt.is_finite() || dt <= 0.0 || !interval.is_finite() || interval <= f32::EPSILON {
        return;
    }

    let target = survivor.as_ref().map(|single| {
        let (entity, transform) = **single;
        (entity, transform.translation)
    });

    for (entity, transform, mut zone) in &mut zones {
        zone.age += dt;
        zone.tick_timer -= dt;

        while zone.tick_timer <= 0.0 {
            zone.tick_timer += interval;
            let Some((target_entity, target_pos)) = target else {
                continue;
            };
            if flat_distance(target_pos, transform.translation) > zone.spec.radius {
                continue;
            }
            damage_writer.write(DamageMessage {
                target: target_entity,
                amount: zone.spec.damage,
                knockback: Vec3::ZERO,
                source: entity,
                kind: zone.spec.kind,
                hit_dir: flat_dir(transform.translation, target_pos).unwrap_or(Vec3::NEG_Z),
            });
        }

        if zone.is_expired() {
            commands.entity(entity).despawn();
        }
    }
}

/// Spawns requested zones, retiring the oldest zone when the cap would be exceeded.
pub fn spawn_hazards(
    mut commands: Commands,
    cfg: Res<HazardConfig>,
    mut requests: MessageReader<HazardSpawnMessage>,
    zones: Query<(Entity, &HazardZone)>,
) {
    if cfg.max_active == 0 {
        return;
    }

    let mut active: Vec<(Entity, f32)> = zones.iter().map(|(entity, zone)| (entity, zone.age)).collect();

    for request in requests.read() {
        if let Err(err) = request.spec.validate() {
            warn!("ignoring invalid hazard spawn: {err}");
            continue;
        }
        if !request.position.is_finite() {
            warn!("ignoring hazard spawn at a non-finite position");
            continue;
        }

        while active.len() >= cfg.max_active {
            let Some((index, (oldest, _))) = active
                .iter()
                .enumerate()
                .max_by(|(_, (_, a)), (_, (_, b))| a.total_cmp(b))
            else {
                break;
            };
            commands.entity(*oldest).despawn();
            active.swap_remove(index);
        }

        let position = Vec3::new(request.position.x, GROUND_Y, request.position.z);
        let zone = commands
            .spawn((
                Name::new("HazardZone"),
                RunEntity,
                DespawnOnExit(GameScene::Arena),
                HazardZone::new(request.spec, cfg.tick_interval),
                Transform::from_translation(position),
                Visibility::default(),
            ))
            .id();
        commands.entity(zone).with_children(|parent| {
            parent.spawn((
                HazardMarker,
                marker_transform(request.spec.radius),
                Visibility::default(),
            ));
        });
        active.push((zone, 0.0));
    }
}

fn flat_distance(a: Vec3, b: Vec3) -> f32 {
    Vec2::new(a.x - b.x, a.z - b.z).length()
}

fn flat_dir(from: Vec3, to: Vec3) -> Option<Vec3> {
    let flat = Vec3::new(to.x - from.x, 0.0, to.z - from.z);
    if flat.length_squared() <= 1e-4 {
        None
    } else {
        Some(flat.normalize())
    }
}
