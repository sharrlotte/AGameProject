use bevy::prelude::*;
use crate::common::{DamageKind, Dying};
use crate::message::messages::DamageMessage;
use crate::world::{PlayerConfig, Survivor};

use crate::enemies::ai::state::Enemy;
use crate::enemies::brain::BrainState;

use super::state::Melee;

/// Resolves wind-up melee: roots the attacker, then damages a target still inside reach.
pub fn enemy_melee(
    time: Res<Time>,
    player_cfg: Res<PlayerConfig>,
    mut damage_writer: MessageWriter<DamageMessage>,
    survivor: Option<Single<(Entity, &Transform), With<Survivor>>>,
    mut attackers: Query<
        (
            Entity,
            &Enemy,
            &Transform,
            &crate::enemies::brain::EnemyBrain,
            &mut Melee,
            Has<Dying>,
        ),
        Without<Survivor>,
    >,
) {
    let dt = time.delta_secs();
    let target = survivor.as_ref().map(|single| {
        let (entity, transform) = **single;
        (entity, transform.translation)
    });

    for (entity, enemy, transform, brain, mut melee, dying) in &mut attackers {
        if dying {
            continue;
        }
        melee.cooldown_timer = (melee.cooldown_timer - dt).max(0.0);

        if melee.is_winding() {
            melee.windup_timer = (melee.windup_timer - dt).max(0.0);
            if melee.windup_timer <= 0.0 {
                melee.cooldown_timer = melee.cooldown;
                if let Some((target_entity, target_pos)) = target {
                    let pos = transform.translation;
                    if flat_distance(target_pos, pos) <= reach(player_cfg.radius, enemy.radius, &melee)
                    {
                        let dir = flat_dir(target_pos - pos).unwrap_or(Vec3::NEG_Z);
                        damage_writer.write(DamageMessage {
                            target: target_entity,
                            amount: melee.damage,
                            knockback: dir * melee.knockback,
                            source: entity,
                            kind: DamageKind::Shock,
                            hit_dir: dir,
                        });
                    }
                }
            }
            continue;
        }

        if brain.state != BrainState::Engage || melee.cooldown_timer > 0.0 {
            continue;
        }
        let Some((target_entity, target_pos)) = target else {
            continue;
        };
        if brain.target != Some(target_entity) {
            continue;
        }
        let pos = transform.translation;
        if flat_distance(target_pos, pos) > reach(player_cfg.radius, enemy.radius, &melee) {
            continue;
        }

        if melee.windup > 0.0 {
            melee.windup_timer = melee.windup;
        } else {
            melee.cooldown_timer = melee.cooldown;
            let dir = flat_dir(target_pos - pos).unwrap_or(Vec3::NEG_Z);
            damage_writer.write(DamageMessage {
                target: target_entity,
                amount: melee.damage,
                knockback: dir * melee.knockback,
                source: entity,
                kind: DamageKind::Shock,
                hit_dir: dir,
            });
        }
    }
}

fn reach(player_radius: f32, enemy_radius: f32, melee: &Melee) -> f32 {
    player_radius + enemy_radius + melee.reach
}

fn flat_distance(a: Vec3, b: Vec3) -> f32 {
    Vec2::new(a.x - b.x, a.z - b.z).length()
}

fn flat_dir(delta: Vec3) -> Option<Vec3> {
    let flat = Vec3::new(delta.x, 0.0, delta.z);
    if flat.length_squared() <= 1e-4 {
        None
    } else {
        Some(flat.normalize())
    }
}
