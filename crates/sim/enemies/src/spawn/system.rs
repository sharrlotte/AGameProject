use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use data::{
    EnemyKind, EnemyTag, GROUP_ENEMY, GROUP_PLAYER, GROUP_PROJECTILE, GROUP_WORLD, RunEntity,
    RunStats, GameScene,
};
use world::{ArenaConfig, Health, PlayerConfig, Survivor};

use crate::ai::state::{Enemy, FacingDirection};
use crate::config::{EnemiesConfig, EnemyArchetypeData};

use super::state::EnemySpawner;

pub fn reset_spawner(mut spawner: ResMut<EnemySpawner>, cfg: Res<EnemiesConfig>) {
    spawner.timer = cfg.initial_spawn_delay;
}

fn pick_edge_spawn_pos(
    survivors: &[Vec3],
    arena_cfg: &ArenaConfig,
    enemies_cfg: &EnemiesConfig,
    aim_height: f32,
) -> Vec3 {
    let edge_limit = (arena_cfg.half_size - enemies_cfg.edge_spawn_inset).max(2.0);
    let span_limit = (edge_limit - 1.5).max(1.0);

    for _ in 0..8 {
        let side = rand::random_range(0..4);
        let offset = rand::random_range(-span_limit..=span_limit);
        let candidate = match side {
            0 => Vec3::new(offset, aim_height, -edge_limit),
            1 => Vec3::new(offset, aim_height, edge_limit),
            2 => Vec3::new(-edge_limit, aim_height, offset),
            _ => Vec3::new(edge_limit, aim_height, offset),
        };

        let far_enough = survivors.iter().all(|s| {
            Vec2::new(candidate.x - s.x, candidate.z - s.z).length()
                >= enemies_cfg.min_spawn_dist_from_survivor
        });
        if far_enough {
            return candidate;
        }
    }

    Vec3::new(edge_limit, aim_height, edge_limit)
}

/// Spawns a single zombie of the given archetype at `spawn_pos`.
pub fn spawn_enemy_entity(
    commands: &mut Commands,
    kind: EnemyKind,
    arch: EnemyArchetypeData,
    spawn_pos: Vec3,
) -> Entity {
    commands
        .spawn((
            (
                Name::new(match kind {
                    EnemyKind::Walker => "Walker",
                    EnemyKind::Runner => "Runner",
                }),
                RunEntity,
                EnemyTag,
                Enemy::from_archetype(kind, arch),
                FacingDirection(Vec3::NEG_Z),
                Health::new(arch.max_hp),
                DespawnOnExit(GameScene::Arena),
            ),
            (
                RigidBody::Dynamic,
                Collider::capsule_y(arch.half_height, arch.radius),
                LockedAxes::TRANSLATION_LOCKED_Y | LockedAxes::ROTATION_LOCKED,
                Velocity::zero(),
                Sleeping::disabled(),
                TransformInterpolation::default(),
                Damping {
                    linear_damping: 0.0,
                    angular_damping: 0.0,
                },
                Friction {
                    coefficient: 0.0,
                    combine_rule: CoefficientCombineRule::Min,
                },
                Restitution {
                    coefficient: 0.0,
                    combine_rule: CoefficientCombineRule::Min,
                },
                CollisionGroups::new(
                    GROUP_ENEMY,
                    GROUP_WORLD | GROUP_PLAYER | GROUP_ENEMY | GROUP_PROJECTILE,
                ),
                Transform::from_translation(spawn_pos),
            ),
        ))
        .id()
}

/// Computes current escalated spawn interval and cap from `RunStats.elapsed_secs`.
pub fn escalated_spawn_params(elapsed_secs: f32, cfg: &EnemiesConfig) -> (f32, usize) {
    let t = if cfg.escalation_duration_secs > 0.0 {
        (elapsed_secs / cfg.escalation_duration_secs).clamp(0.0, 1.0)
    } else {
        1.0
    };
    let interval =
        cfg.spawn_interval_start + (cfg.spawn_interval_min - cfg.spawn_interval_start) * t;
    let max_active =
        cfg.max_active_start as f32 + (cfg.max_active_end as f32 - cfg.max_active_start as f32) * t;
    (interval.max(0.1), max_active.round().max(1.0) as usize)
}

/// Spawns Walkers and Runners along the arena edges, escalating over run time.
#[allow(clippy::too_many_arguments)]
pub fn spawn_enemies(
    mut commands: Commands,
    time: Res<Time>,
    arena_cfg: Res<ArenaConfig>,
    player_cfg: Res<PlayerConfig>,
    enemies_cfg: Res<EnemiesConfig>,
    stats: Res<RunStats>,
    mut spawner: ResMut<EnemySpawner>,
    existing_enemies: Query<(), With<Enemy>>,
    survivors: Query<&Transform, With<Survivor>>,
) {
    spawner.timer -= time.delta_secs();
    if spawner.timer > 0.0 {
        return;
    }

    let (interval, max_active) = escalated_spawn_params(stats.elapsed_secs, &enemies_cfg);
    spawner.timer = interval;

    if existing_enemies.iter().count() >= max_active {
        return;
    }

    let survivor_positions: Vec<Vec3> = survivors.iter().map(|t| t.translation).collect();
    if survivor_positions.is_empty() {
        return;
    }

    let (kind, arch) = if rand::random_range(0.0..1.0) < enemies_cfg.walker_spawn_weight {
        (EnemyKind::Walker, enemies_cfg.walker)
    } else {
        (EnemyKind::Runner, enemies_cfg.runner)
    };
    let spawn_pos = pick_edge_spawn_pos(
        &survivor_positions,
        &arena_cfg,
        &enemies_cfg,
        player_cfg.aim_height,
    );
    spawn_enemy_entity(&mut commands, kind, arch, spawn_pos);
}
