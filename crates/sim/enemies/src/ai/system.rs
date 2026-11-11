use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use data::{ConfigHotReloadedMessage, DamageMessage, EnemyKind};
use world::{PlayerConfig, Survivor};

use crate::config::EnemiesConfig;

use super::state::{Enemy, FacingDirection};

pub fn freeze_enemies_on_game_over(mut enemies: Query<&mut Velocity, With<Enemy>>) {
    for mut vel in &mut enemies {
        vel.linear = Vec3::ZERO;
        vel.angular = Vec3::ZERO;
    }
}

pub fn sync_enemies_hot_reload(
    mut reloaded: MessageReader<ConfigHotReloadedMessage>,
    cfg: Res<EnemiesConfig>,
    mut enemies: Query<&mut Enemy>,
) {
    if reloaded.read().next().is_none() || !cfg.is_changed() {
        return;
    }

    for mut enemy in &mut enemies {
        let arch = match enemy.kind {
            EnemyKind::Walker => cfg.walker,
            EnemyKind::Runner => cfg.runner,
        };
        enemy.move_speed = arch.move_speed;
        enemy.attack_damage = arch.attack_damage;
        enemy.attack_cooldown = arch.attack_cooldown;
        enemy.mass = arch.mass;
    }
}

/// Moves zombies directly toward the nearest survivor and applies contact damage on cooldown.
pub fn enemy_chase_and_attack(
    time: Res<Time>,
    player_cfg: Res<PlayerConfig>,
    enemies_cfg: Res<EnemiesConfig>,
    mut damage_writer: MessageWriter<DamageMessage>,
    survivors: Query<(Entity, &Transform), With<Survivor>>,
    mut enemies: Query<
        (
            Entity,
            &mut Enemy,
            &mut Velocity,
            &Transform,
            &mut FacingDirection,
        ),
        Without<Survivor>,
    >,
) {
    let survivor_list: Vec<(Entity, Vec3)> =
        survivors.iter().map(|(e, t)| (e, t.translation)).collect();
    if survivor_list.is_empty() {
        for (_, _, mut vel, _, _) in &mut enemies {
            vel.linear = Vec3::ZERO;
        }
        return;
    }

    let dt = time.delta_secs();
    let decay = (1.0 - enemies_cfg.knockback_decay_rate * dt).max(0.0);

    for (enemy_entity, mut enemy, mut vel, transform, mut facing) in &mut enemies {
        enemy.attack_timer = (enemy.attack_timer - dt).max(0.0);
        enemy.knockback_vel *= decay;

        let pos = transform.translation;
        let Some(&(target_entity, target_pos)) = survivor_list.iter().min_by(|a, b| {
            let da = Vec2::new(a.1.x - pos.x, a.1.z - pos.z).length_squared();
            let db = Vec2::new(b.1.x - pos.x, b.1.z - pos.z).length_squared();
            da.total_cmp(&db)
        }) else {
            continue;
        };

        let to_target = Vec3::new(target_pos.x - pos.x, 0.0, target_pos.z - pos.z);
        let dist = to_target.length();
        let chase_dir = if dist > 1e-4 {
            to_target / dist
        } else {
            Vec3::ZERO
        };

        vel.linear = chase_dir * enemy.move_speed + enemy.knockback_vel;
        if chase_dir.length_squared() > 1e-4 {
            facing.0 = chase_dir;
        }

        let reach = player_cfg.radius + enemy.radius + enemies_cfg.contact_reach_buffer;
        if dist <= reach && enemy.attack_timer <= 0.0 {
            enemy.attack_timer = enemy.attack_cooldown;
            damage_writer.write(DamageMessage {
                target: target_entity,
                amount: enemy.attack_damage,
                knockback: Vec3::ZERO,
                source: enemy_entity,
            });
        }
    }
}
