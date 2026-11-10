use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use data::{
    ArenaConfig, ConfigHotReloadedMessage, DamageMessage, EnemiesConfig, EnemyArchetypeData,
    EnemyKilledMessage, EnemyKind, EnemyTag, GROUP_ENEMY, GROUP_PLAYER, GROUP_PROJECTILE,
    GROUP_WORLD, GameState, PlayerConfig, RunEntity, RunStats, SimPostPhysicsSet, SimUpdateSet,
};
use world::{Health, Survivor};

#[derive(Component, Debug, Clone, Copy)]
pub struct FacingDirection(pub Vec3);

#[derive(Component, Debug, Clone)]
pub struct Enemy {
    pub kind: EnemyKind,
    pub move_speed: f32,
    pub attack_damage: f32,
    pub attack_cooldown: f32,
    pub attack_timer: f32,
    pub radius: f32,
    pub half_height: f32,
    pub mass: f32,
    pub knockback_vel: Vec3,
}

impl Enemy {
    pub fn from_archetype(kind: EnemyKind, arch: EnemyArchetypeData) -> Self {
        Self {
            kind,
            move_speed: arch.move_speed,
            attack_damage: arch.attack_damage,
            attack_cooldown: arch.attack_cooldown,
            attack_timer: 0.0,
            radius: arch.radius,
            half_height: arch.half_height,
            mass: arch.mass,
            knockback_vel: Vec3::ZERO,
        }
    }
}

#[derive(Resource, Debug, Default)]
pub struct EnemySpawner {
    pub timer: f32,
}

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EnemySpawner>()
            .add_systems(OnEnter(GameState::Playing), reset_spawner)
            .add_systems(OnEnter(GameState::GameOver), freeze_enemies_on_game_over)
            .add_systems(
                Update,
                (
                    sync_enemies_hot_reload,
                    spawn_enemies,
                    enemy_chase_and_attack,
                )
                    .chain()
                    .in_set(SimUpdateSet)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                PostUpdate,
                apply_enemy_damage
                    .in_set(SimPostPhysicsSet)
                    .after(PhysicsSet::Writeback)
                    .before(TransformSystems::Propagate)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn reset_spawner(mut spawner: ResMut<EnemySpawner>, cfg: Res<EnemiesConfig>) {
    spawner.timer = cfg.initial_spawn_delay;
}

fn freeze_enemies_on_game_over(mut enemies: Query<&mut Velocity, With<Enemy>>) {
    for mut vel in &mut enemies {
        vel.linear = Vec3::ZERO;
        vel.angular = Vec3::ZERO;
    }
}

fn sync_enemies_hot_reload(
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
    let max_active = cfg.max_active_start as f32
        + (cfg.max_active_end as f32 - cfg.max_active_start as f32) * t;
    (interval.max(0.1), max_active.round().max(1.0) as usize)
}

/// Spawns Walkers and Runners along the arena edges, escalating over run time.
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
    let spawn_pos =
        pick_edge_spawn_pos(&survivor_positions, &arena_cfg, &enemies_cfg, player_cfg.aim_height);
    spawn_enemy_entity(&mut commands, kind, arch, spawn_pos);
}

/// Moves zombies directly toward the nearest survivor and applies contact damage on cooldown.
pub fn enemy_chase_and_attack(
    time: Res<Time>,
    player_cfg: Res<PlayerConfig>,
    enemies_cfg: Res<EnemiesConfig>,
    mut damage_writer: MessageWriter<DamageMessage>,
    survivors: Query<(Entity, &Transform), With<Survivor>>,
    mut enemies: Query<
        (Entity, &mut Enemy, &mut Velocity, &Transform, &mut FacingDirection),
        Without<Survivor>,
    >,
) {
    let survivor_list: Vec<(Entity, Vec3)> = survivors
        .iter()
        .map(|(e, t)| (e, t.translation))
        .collect();
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

#[cfg(test)]
mod tests;
