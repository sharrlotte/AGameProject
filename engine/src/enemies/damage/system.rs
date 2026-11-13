use bevy::prelude::*;
use crate::message::messages::{DamageMessage, EnemyKilledMessage};
use crate::world::Health;

use crate::enemies::ai::state::Enemy;

/// Applies projectile damage and knockback impulses to enemies, emitting `EnemyKilledMessage` at 0 HP.
pub fn apply_enemy_damage(
    mut commands: Commands,
    mut damage_reader: MessageReader<DamageMessage>,
    mut killed_writer: MessageWriter<EnemyKilledMessage>,
    mut enemies: Query<(&mut Enemy, &mut Health, &Transform)>,
) {
    for msg in damage_reader.read() {
        let Ok((mut enemy, mut health, transform)) = enemies.get_mut(msg.target) else {
            continue;
        };
        if health.current <= 0.0 {
            continue;
        }

        health.current = (health.current - msg.amount).max(0.0);
        let mass = enemy.mass.max(0.1);
        enemy.knockback_vel += msg.knockback / mass;

        if health.current <= 0.0 {
            killed_writer.write(EnemyKilledMessage {
                entity: msg.target,
                position: transform.translation,
                kind: enemy.kind,
            });
            commands.entity(msg.target).despawn();
        }
    }
}

