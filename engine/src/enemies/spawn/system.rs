use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::common::{
    Armor, EnemyKind, GROUP_ENEMY, GROUP_PLAYER, GROUP_PROJECTILE, GROUP_WORLD, RunEntity, GameScene,
};
use crate::message::resources::RunStats;
use crate::world::arena::EnemySpawnerMarker;
use crate::world::{ArenaConfig, Health, Survivor};

use crate::enemies::ai::state::{Enemy, FacingDirection};
use crate::enemies::brain::EnemyBrain;
use crate::enemies::detonate::Fuse;
use crate::enemies::melee::Melee;
use crate::enemies::ranged::Ranged;
use crate::config::{EnemiesConfig, EnemyArchetypeData};

use super::state::EnemySpawner;

pub fn reset_spawner(mut spawner: ResMut<EnemySpawner>, cfg: Res<EnemiesConfig>) {
    spawner.timer = cfg.initial_spawn_delay;
}

fn pick_edge_spawn_pos(
    survivors: &[Vec3],
    arena_cfg: &ArenaConfig,
    enemies_cfg: &EnemiesConfig,
    spawn_y: f32,
) -> Vec3 {
    let edge_limit = (arena_cfg.half_size - enemies_cfg.edge_spawn_inset).max(2.0);
    let span_limit = (edge_limit - 1.5).max(1.0);

    for _ in 0..8 {
        let side = rand::random_range(0..4);
        let offset = rand::random_range(-span_limit..=span_limit);
        let candidate = match side {
            0 => Vec3::new(offset, spawn_y, -edge_limit),
            1 => Vec3::new(offset, spawn_y, edge_limit),
            2 => Vec3::new(-edge_limit, spawn_y, offset),
            _ => Vec3::new(edge_limit, spawn_y, offset),
        };

        let far_enough = survivors.iter().all(|s| {
            Vec2::new(candidate.x - s.x, candidate.z - s.z).length()
                >= enemies_cfg.min_spawn_dist_from_survivor
        });
        if far_enough {
            return candidate;
        }
    }

    Vec3::new(edge_limit, spawn_y, edge_limit)
}

/// Spawns one archetype at `spawn_pos` with the behaviors authored in its roster entry.
pub fn spawn_enemy_entity(
    commands: &mut Commands,
    arch: &EnemyArchetypeData,
    spawn_pos: Vec3,
) -> Entity {
    let entity = commands
        .spawn((
            (
                Name::new(arch.kind.label()),
                RunEntity,
                Enemy::from_archetype(arch),
                EnemyBrain::default(),
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
        .id();

    if let Some(spec) = arch.melee() {
        commands.entity(entity).insert(Melee::from_spec(spec));
    }
    if let Some(spec) = arch.armor() {
        commands.entity(entity).insert(Armor {
            front_mult: spec.front_mult,
            back_mult: spec.back_mult,
            arc_deg: spec.arc_deg,
            bypass: spec.bypass,
        });
    }
    if let Some(spec) = arch.ranged() {
        commands.entity(entity).insert(Ranged::from_spec(spec));
    }
    if let Some(spec) = arch.detonate() {
        commands.entity(entity).insert(Fuse::from_spec(spec));
    }
    entity
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

fn count_kind(enemies: &Query<&Enemy>, kind: EnemyKind) -> usize {
    enemies.iter().filter(|enemy| enemy.kind == kind).count()
}

fn collect_counts(enemies: &Query<&Enemy>, cfg: &EnemiesConfig) -> Vec<(EnemyKind, usize)> {
    cfg.archetypes
        .iter()
        .map(|arch| (arch.kind, count_kind(enemies, arch.kind)))
        .collect()
}

/// Archetypes that are unlocked, weighted, under their own cap, and within the global cap.
pub fn eligible_archetypes<'a>(
    cfg: &'a EnemiesConfig,
    elapsed_secs: f32,
    counts: &[(EnemyKind, usize)],
    global_active: usize,
    global_cap: usize,
) -> Vec<&'a EnemyArchetypeData> {
    if global_active >= global_cap {
        return Vec::new();
    }
    cfg.archetypes
        .iter()
        .filter(|arch| elapsed_secs >= arch.spawn.unlock_secs && arch.spawn.weight > 0.0)
        .filter(|arch| {
            counts
                .iter()
                .find(|(kind, _)| *kind == arch.kind)
                .is_none_or(|(_, count)| *count < arch.spawn.max_active)
        })
        .collect()
}

/// Picks one eligible archetype from a unit roll in `[0, 1)`.
pub fn pick_archetype<'a>(
    eligible: &[&'a EnemyArchetypeData],
    roll: f32,
) -> Option<&'a EnemyArchetypeData> {
    let total: f32 = eligible.iter().map(|arch| arch.spawn.weight).sum();
    if total <= 0.0 {
        return None;
    }
    let mut cursor = roll.clamp(0.0, 1.0) * total;
    for arch in eligible {
        if cursor < arch.spawn.weight {
            return Some(arch);
        }
        cursor -= arch.spawn.weight;
    }
    eligible.last().copied()
}

/// Rolls one eligible archetype and spawns it along the arena edge.
pub fn spawn_enemies(
    mut commands: Commands,
    time: Res<Time>,
    arena_cfg: Res<ArenaConfig>,
    enemies_cfg: Res<EnemiesConfig>,
    stats: Res<RunStats>,
    mut spawner: ResMut<EnemySpawner>,
    existing_enemies: Query<&Enemy>,
    survivors: Query<&Transform, With<Survivor>>,
    spawner_markers: Query<(&Transform, &EnemySpawnerMarker)>,
) {
    spawner.timer -= time.delta_secs();
    if spawner.timer > 0.0 {
        return;
    }

    let (interval, max_active) = escalated_spawn_params(stats.elapsed_secs, &enemies_cfg);
    spawner.timer = interval;

    let counts = collect_counts(&existing_enemies, &enemies_cfg);
    let active = existing_enemies.iter().count();
    let eligible = eligible_archetypes(
        &enemies_cfg,
        stats.elapsed_secs,
        &counts,
        active,
        max_active,
    );
    let Some(chosen) = pick_archetype(&eligible, rand::random_range(0.0..1.0)) else {
        return;
    };

    let survivor_positions: Vec<Vec3> = survivors.iter().map(|t| t.translation).collect();
    if survivor_positions.is_empty() {
        return;
    }

    let marker_candidates: Vec<Vec3> = spawner_markers
        .iter()
        .filter(|(_, marker)| {
            marker.kind_filter.as_ref().is_none_or(|filter| {
                filter.eq_ignore_ascii_case(chosen.kind.label())
            })
        })
        .map(|(t, _)| Vec3::new(t.translation.x, chosen.collider_center_y(), t.translation.z))
        .filter(|pos| {
            survivor_positions.iter().all(|s| {
                Vec2::new(pos.x - s.x, pos.z - s.z).length()
                    >= enemies_cfg.min_spawn_dist_from_survivor
            })
        })
        .collect();

    let spawn_pos = if !marker_candidates.is_empty() {
        let idx = rand::random_range(0..marker_candidates.len());
        marker_candidates[idx]
    } else {
        pick_edge_spawn_pos(
            &survivor_positions,
            &arena_cfg,
            &enemies_cfg,
            chosen.collider_center_y(),
        )
    };
    spawn_enemy_entity(&mut commands, chosen, spawn_pos);
}
