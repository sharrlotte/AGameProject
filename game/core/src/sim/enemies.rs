use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use super::arena::ARENA_HALF_SIZE;
use super::player::{AIM_HEIGHT, Health, PLAYER_RADIUS, Survivor};
use super::{DamageMessage, EnemyKilledMessage, GROUP_ENEMY, GROUP_PLAYER, GROUP_PROJECTILE, GROUP_WORLD, RunEntity, RunStats};

const SPAWN_INTERVAL_SECS: f32 = 1.5;
const MAX_ACTIVE_ENEMIES: usize = 40;
const EDGE_SPAWN_INSET: f32 = 2.2;
const MIN_SPAWN_DIST_FROM_SURVIVOR: f32 = 12.0;
const KNOCKBACK_DECAY_RATE: f32 = 10.0;
const CONTACT_REACH_BUFFER: f32 = 0.25;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyKind {
    Walker,
    Runner,
}

#[derive(Debug, Clone, Copy)]
pub struct EnemyArchetype {
    pub kind: EnemyKind,
    pub max_hp: f32,
    pub move_speed: f32,
    pub attack_damage: f32,
    pub attack_cooldown: f32,
    pub radius: f32,
    pub half_height: f32,
    pub mass: f32,
}

pub const WALKER_ARCHETYPE: EnemyArchetype = EnemyArchetype {
    kind: EnemyKind::Walker,
    max_hp: 75.0,
    move_speed: 4.2,
    attack_damage: 15.0,
    attack_cooldown: 0.9,
    radius: 0.45,
    half_height: 0.45,
    mass: 1.0,
};

pub const RUNNER_ARCHETYPE: EnemyArchetype = EnemyArchetype {
    kind: EnemyKind::Runner,
    max_hp: 35.0,
    move_speed: 7.8,
    attack_damage: 10.0,
    attack_cooldown: 0.7,
    radius: 0.35,
    half_height: 0.40,
    mass: 0.65,
};

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
    pub fn from_archetype(arch: EnemyArchetype) -> Self {
        Self {
            kind: arch.kind,
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

#[derive(Resource, Debug)]
pub struct EnemySpawner {
    pub timer: f32,
    pub interval: f32,
    pub max_active: usize,
}

impl Default for EnemySpawner {
    fn default() -> Self {
        Self {
            timer: 0.8,
            interval: SPAWN_INTERVAL_SECS,
            max_active: MAX_ACTIVE_ENEMIES,
        }
    }
}

fn pick_edge_spawn_pos(survivors: &[Vec3]) -> Vec3 {
    let edge_limit = ARENA_HALF_SIZE - EDGE_SPAWN_INSET;
    let span_limit = edge_limit - 1.5;

    for _ in 0..8 {
        let side = rand::random_range(0..4);
        let offset = rand::random_range(-span_limit..=span_limit);
        let candidate = match side {
            0 => Vec3::new(offset, AIM_HEIGHT, -edge_limit),
            1 => Vec3::new(offset, AIM_HEIGHT, edge_limit),
            2 => Vec3::new(-edge_limit, AIM_HEIGHT, offset),
            _ => Vec3::new(edge_limit, AIM_HEIGHT, offset),
        };

        let far_enough = survivors.iter().all(|s| {
            Vec2::new(candidate.x - s.x, candidate.z - s.z).length() >= MIN_SPAWN_DIST_FROM_SURVIVOR
        });
        if far_enough {
            return candidate;
        }
    }

    Vec3::new(edge_limit, AIM_HEIGHT, -edge_limit)
}

/// Spawns a single zombie of the given archetype at `spawn_pos`.
pub fn spawn_enemy_entity(
    commands: &mut Commands,
    arch: EnemyArchetype,
    spawn_pos: Vec3,
) -> Entity {
    commands
        .spawn((
            (
                Name::new(match arch.kind {
                    EnemyKind::Walker => "Walker",
                    EnemyKind::Runner => "Runner",
                }),
                RunEntity,
                Enemy::from_archetype(arch),
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

/// Spawns Walkers and Runners periodically along the edges of the arena.
pub fn spawn_enemies(
    mut commands: Commands,
    time: Res<Time>,
    mut spawner: ResMut<EnemySpawner>,
    existing_enemies: Query<(), With<Enemy>>,
    survivors: Query<&Transform, With<Survivor>>,
) {
    spawner.timer -= time.delta_secs();
    if spawner.timer > 0.0 {
        return;
    }
    spawner.timer = spawner.interval;

    if existing_enemies.iter().count() >= spawner.max_active {
        return;
    }

    let survivor_positions: Vec<Vec3> = survivors.iter().map(|t| t.translation).collect();
    if survivor_positions.is_empty() {
        return;
    }

    let arch = if rand::random_range(0.0..1.0) < 0.7 {
        WALKER_ARCHETYPE
    } else {
        RUNNER_ARCHETYPE
    };
    let spawn_pos = pick_edge_spawn_pos(&survivor_positions);
    spawn_enemy_entity(&mut commands, arch, spawn_pos);
}

/// Moves zombies directly toward the nearest survivor and applies contact damage on cooldown.
pub fn enemy_chase_and_attack(
    time: Res<Time>,
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
    let decay = (1.0 - KNOCKBACK_DECAY_RATE * dt).max(0.0);

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

        let reach = PLAYER_RADIUS + enemy.radius + CONTACT_REACH_BUFFER;
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

/// Applies projectile damage and knockback impulses to enemies, despawning them at 0 HP.
pub fn apply_enemy_damage(
    mut commands: Commands,
    mut damage_reader: MessageReader<DamageMessage>,
    mut killed_writer: MessageWriter<EnemyKilledMessage>,
    mut stats: ResMut<RunStats>,
    mut enemies: Query<(&mut Enemy, &mut Health)>,
) {
    for msg in damage_reader.read() {
        let Ok((mut enemy, mut health)) = enemies.get_mut(msg.target) else {
            continue;
        };
        if health.current <= 0.0 {
            continue;
        }

        health.current = (health.current - msg.amount).max(0.0);
        let mass = enemy.mass.max(0.1);
        enemy.knockback_vel += msg.knockback / mass;

        if health.current <= 0.0 {
            stats.kills += 1;
            killed_writer.write(EnemyKilledMessage { entity: msg.target });
            commands.entity(msg.target).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn enemy_damage_applies_knockback_and_despawns_at_zero() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<RunStats>()
            .insert_resource(Time::<Fixed>::from_hz(60.0))
            .add_message::<DamageMessage>()
            .add_message::<EnemyKilledMessage>()
            .add_systems(FixedUpdate, apply_enemy_damage);

        let enemy = app
            .world_mut()
            .spawn((
                Enemy::from_archetype(RUNNER_ARCHETYPE),
                Health::new(RUNNER_ARCHETYPE.max_hp),
            ))
            .id();

        app.world_mut().write_message(DamageMessage {
            target: enemy,
            amount: 15.0,
            knockback: Vec3::new(0.0, 0.0, -5.0),
            source: Entity::PLACEHOLDER,
        });
        app.world_mut().run_schedule(FixedUpdate);

        let hp = app.world().get::<Health>(enemy).unwrap().current;
        assert!((hp - 20.0).abs() < 0.01);
        assert!(app.world().get::<Enemy>(enemy).unwrap().knockback_vel.z < -1.0);

        app.world_mut().write_message(DamageMessage {
            target: enemy,
            amount: 25.0,
            knockback: Vec3::ZERO,
            source: Entity::PLACEHOLDER,
        });
        app.world_mut().run_schedule(FixedUpdate);

        assert!(app.world().get_entity(enemy).is_err());
        assert_eq!(app.world().resource::<RunStats>().kills, 1);
    }

    #[test]
    fn enemy_chases_and_deals_contact_damage_on_cooldown() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
                Duration::from_secs_f32(1.0 / 60.0),
            ))
            .add_message::<DamageMessage>()
            .add_systems(Update, enemy_chase_and_attack);

        let survivor = app
            .world_mut()
            .spawn((Survivor, Transform::from_xyz(0.0, AIM_HEIGHT, 0.0)))
            .id();
        let zombie = app
            .world_mut()
            .spawn((
                Enemy::from_archetype(WALKER_ARCHETYPE),
                FacingDirection(Vec3::NEG_Z),
                Velocity::zero(),
                Transform::from_xyz(0.0, AIM_HEIGHT, 0.8),
            ))
            .id();

        app.update();

        let vel = app.world().get::<Velocity>(zombie).unwrap().linear;
        assert!(vel.z < -1.0, "expected chase velocity toward -Z, got {vel:?}");

        let messages = app.world().resource::<Messages<DamageMessage>>();
        let mut reader = messages.get_cursor();
        let hits: Vec<_> = reader.read(messages).copied().collect();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].target, survivor);
        assert_eq!(hits[0].amount, WALKER_ARCHETYPE.attack_damage);
    }

    #[test]
    fn walker_and_runner_move_with_rapier_when_player_stationary() {
        use bevy::ecs::system::RunSystemOnce;

        use crate::sim::arena::spawn_arena_physics;
        use crate::sim::player::spawn_player;

        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            bevy::asset::AssetPlugin::default(),
            bevy::mesh::MeshPlugin,
            bevy::world_serialization::WorldSerializationPlugin,
            bevy::transform::TransformPlugin,
            bevy::state::app::StatesPlugin,
            RapierPhysicsPlugin::<NoUserData>::default(),
        ))
        .insert_resource(TimestepMode::Interpolated {
            dt: 1.0 / 60.0,
            time_scale: 1.0,
            substeps: 1,
        })
        .init_state::<crate::GameState>()
        .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            Duration::from_secs_f32(1.0 / 60.0),
        ))
        .add_message::<DamageMessage>()
        .add_systems(Startup, (spawn_arena_physics, spawn_player))
        .add_systems(
            Update,
            (
                crate::sim::player::player_movement_and_aim,
                enemy_chase_and_attack,
            )
                .chain(),
        );

        app.update();

        let (walker, runner) = app
            .world_mut()
            .run_system_once(|mut commands: Commands| {
                let w = spawn_enemy_entity(
                    &mut commands,
                    WALKER_ARCHETYPE,
                    Vec3::new(10.0, AIM_HEIGHT, 0.0),
                );
                let r = spawn_enemy_entity(
                    &mut commands,
                    RUNNER_ARCHETYPE,
                    Vec3::new(-10.0, AIM_HEIGHT, 0.0),
                );
                (w, r)
            })
            .expect("spawn test enemies");

        for _ in 0..60 {
            app.update();
        }

        let walker_pos = app.world().get::<Transform>(walker).unwrap().translation;
        let runner_pos = app.world().get::<Transform>(runner).unwrap().translation;
        assert!(walker_pos.x < 7.0, "walker failed to move: {walker_pos:?}");
        assert!(runner_pos.x > -4.0, "runner failed to move: {runner_pos:?}");
    }
}
