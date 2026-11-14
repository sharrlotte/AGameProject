use bevy::prelude::*;

use crate::common::{DamageKind, Dying};
use crate::message::messages::{DamageMessage, DetonationMessage, EnemyKilledMessage};
use crate::world::{Health, Survivor};

use crate::enemies::ai::state::Enemy;
use crate::enemies::brain::{BrainState, EnemyBrain};

use super::state::{Detonated, Fuse};

/// Arms the irreversible fuse on engagement and counts it down.
pub fn update_fuses(
    time: Res<Time>,
    survivor: Option<Single<&Transform, With<Survivor>>>,
    mut fuses: Query<(&mut Fuse, &Transform, &EnemyBrain, Has<Dying>)>,
) {
    let dt = time.delta_secs();
    let survivor_pos = survivor.as_ref().map(|single| single.translation);

    for (mut fuse, transform, brain, dying) in &mut fuses {
        if dying {
            continue;
        }
        if !fuse.armed
            && brain.state == BrainState::Engage
            && let Some(target) = survivor_pos
            && flat_distance(target, transform.translation) <= fuse.trigger_radius
        {
            fuse.armed = true;
            fuse.timer = fuse.fuse;
        }
        if fuse.armed {
            fuse.timer = (fuse.timer - dt).max(0.0);
        }
    }
}

/// Detonates armed fuses and dying detonators once, before dead entities are cleaned up.
#[allow(clippy::type_complexity)]
pub fn resolve_detonations(
    mut commands: Commands,
    mut damage_writer: MessageWriter<DamageMessage>,
    mut killed_writer: MessageWriter<EnemyKilledMessage>,
    mut blast_writer: MessageWriter<DetonationMessage>,
    survivor: Option<Single<(Entity, &Transform), With<Survivor>>>,
    mut detonators: Query<
        (
            Entity,
            &Transform,
            &Enemy,
            &mut Fuse,
            &mut Health,
            Has<Dying>,
            Has<Detonated>,
        ),
        Without<Survivor>,
    >,
    targets: Query<(Entity, &Transform), (With<Enemy>, Without<Survivor>, Without<Dying>)>,
) {
    let mut blasts: Vec<(Entity, Vec3, f32, f32, f32, bool)> = Vec::new();
    for (entity, transform, _, fuse, _, dying, detonated) in &detonators {
        if detonated {
            continue;
        }
        let expired = fuse.armed && fuse.timer <= 0.0;
        if !dying && !expired {
            continue;
        }
        blasts.push((
            entity,
            transform.translation,
            fuse.damage,
            fuse.radius,
            fuse.knockback,
            !dying,
        ));
    }

    let survivor_target = survivor.as_ref().map(|single| {
        let (entity, transform) = **single;
        (entity, transform.translation)
    });

    for (entity, center, damage, radius, knockback, proximity_kill) in blasts {
        if let Ok((_, _, enemy, _, mut health, _, _)) = detonators.get_mut(entity) {
            health.current = 0.0;
            if proximity_kill {
                killed_writer.write(EnemyKilledMessage {
                    entity,
                    position: center,
                    kind: enemy.kind,
                    scrap: enemy.scrap_value,
                });
            }
        }
        commands.entity(entity).insert((Dying, Detonated));
        blast_writer.write(DetonationMessage {
            position: center,
            radius,
        });

        if let Some((target_entity, target_pos)) = survivor_target {
            apply_blast(
                &mut damage_writer,
                entity,
                center,
                target_entity,
                target_pos,
                damage,
                radius,
                knockback,
            );
        }

        for (target_entity, target_transform) in &targets {
            if target_entity == entity {
                continue;
            }
            apply_blast(
                &mut damage_writer,
                entity,
                center,
                target_entity,
                target_transform.translation,
                damage,
                radius,
                knockback,
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_blast(
    damage_writer: &mut MessageWriter<DamageMessage>,
    source: Entity,
    center: Vec3,
    target: Entity,
    target_pos: Vec3,
    damage: f32,
    radius: f32,
    knockback: f32,
) {
    let distance = flat_distance(center, target_pos);
    let falloff = blast_falloff(distance, radius);
    if falloff <= 0.0 {
        return;
    }
    let dir = flat_dir(center, target_pos).unwrap_or(Vec3::ZERO);
    damage_writer.write(DamageMessage {
        target,
        amount: damage * falloff,
        knockback: dir * knockback * falloff,
        source,
        kind: DamageKind::Explosive,
        hit_dir: dir,
    });
}

pub fn blast_falloff(distance: f32, radius: f32) -> f32 {
    if !distance.is_finite() || !radius.is_finite() || radius <= 0.0 {
        return 0.0;
    }
    (1.0 - distance / radius).clamp(0.0, 1.0)
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
