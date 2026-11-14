use bevy::prelude::*;
use crate::common::{Armor, Dying};
use crate::config::EnemiesConfig;
use crate::message::messages::{DamageMessage, EnemyKilledMessage};
use crate::world::{Health, Survivor};

use crate::enemies::ai::state::{Enemy, FacingDirection};
use crate::enemies::brain::EnemyBrain;

/// Applies armor mitigation, knockback, aggro, and stagger, then marks lethal targets `Dying`.
#[allow(clippy::type_complexity)]
pub fn apply_enemy_damage(
    mut commands: Commands,
    cfg: Res<EnemiesConfig>,
    mut damage_reader: MessageReader<DamageMessage>,
    mut killed_writer: MessageWriter<EnemyKilledMessage>,
    survivors: Query<&Transform, With<Survivor>>,
    mut enemies: Query<(
        &mut Enemy,
        &mut Health,
        &Transform,
        &FacingDirection,
        Option<&Armor>,
        Option<&mut EnemyBrain>,
        Has<Dying>,
    )>,
) {
    for msg in damage_reader.read() {
        let Ok((mut enemy, mut health, transform, facing, armor, brain, dying)) =
            enemies.get_mut(msg.target)
        else {
            continue;
        };
        if dying || health.current <= 0.0 {
            continue;
        }

        let mitigated = armor.map_or(msg.amount, |armor| {
            msg.amount * armor.multiplier(facing.0, msg.hit_dir, msg.kind)
        });
        health.current = (health.current - mitigated.max(0.0)).max(0.0);
        let mass = enemy.mass.max(0.1);
        enemy.knockback_vel += msg.knockback / mass;

        if let Some(mut brain) = brain {
            if let Ok(source) = survivors.get(msg.source) {
                brain.engage(msg.source, source.translation);
            }
            if enemy.knockback_vel.length() > cfg.stagger_threshold {
                brain.stagger(cfg.stagger_secs);
            }
        }

        if health.current <= 0.0 {
            commands.entity(msg.target).insert(Dying);
            killed_writer.write(EnemyKilledMessage {
                entity: msg.target,
                position: transform.translation,
                kind: enemy.kind,
                scrap: enemy.scrap_value,
            });
        }
    }
}
