use std::time::Duration;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use bevy_rapier3d::prelude::*;

use crate::common::{EnemyKind, GROUP_ENEMY, GROUP_WORLD};
use crate::config::EnemiesConfig;
use crate::world::{PlayerConfig, Survivor};

use crate::enemies::ai::state::{Enemy, FacingDirection};
use crate::enemies::ai::system::steer_enemies;
use crate::enemies::brain::{BrainState, EnemyBrain};
use crate::enemies::ranged::state::{AcidProjectile, Ranged};
use crate::enemies::ranged::system::{
    enemy_ranged, launch_velocity, spawn_acid_projectile,
};

fn config() -> EnemiesConfig {
    let config: EnemiesConfig = crate::assert_ron_config_parses!(EnemiesConfig);
    config.validate().expect("authored roster is valid");
    config
}

fn spitter_spec(config: &EnemiesConfig) -> crate::config::RangedSpec {
    config
        .archetype(EnemyKind::Spitter)
        .expect("authored spitter")
        .ranged()
        .expect("spitter ranged behavior")
}

fn ranged_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(0.1)))
        .insert_resource(config())
        .init_resource::<PlayerConfig>()
        .add_systems(Update, enemy_ranged);
    app
}

fn spawn_survivor(app: &mut App, at: Vec3) -> Entity {
    app.world_mut()
        .spawn((Survivor, Transform::from_translation(at)))
        .id()
}

fn spawn_spitter(app: &mut App, at: Vec3, target: Entity) -> Entity {
    let config = app.world().resource::<EnemiesConfig>().clone();
    let arch = config
        .archetype(EnemyKind::Spitter)
        .expect("authored spitter")
        .clone();
    let spec = spitter_spec(&config);
    app.world_mut()
        .spawn((
            Enemy::from_archetype(&arch),
            Ranged::from_spec(spec),
            EnemyBrain {
                state: BrainState::Engage,
                target: Some(target),
                last_known: Some(Vec3::ZERO),
                ..default()
            },
            Transform::from_translation(at),
        ))
        .id()
}

fn projectile_count(app: &mut App) -> usize {
    let world = app.world_mut();
    world
        .query_filtered::<Entity, With<AcidProjectile>>()
        .iter(world)
        .count()
}

#[test]
fn windup_telegraphs_then_fires_one_projectile() {
    let mut app = ranged_app();
    let survivor = spawn_survivor(&mut app, Vec3::new(0.0, 0.9, 0.0));
    let spitter = spawn_spitter(&mut app, Vec3::new(0.0, 1.0, 8.0), survivor);

    app.update();
    let ranged = *app.world().get::<Ranged>(spitter).expect("ranged");
    assert!(ranged.is_winding(), "attack must telegraph");
    assert_eq!(projectile_count(&mut app), 0);

    for _ in 0..5 {
        app.update();
    }
    assert_eq!(projectile_count(&mut app), 0, "no shot during wind-up");

    app.update();
    assert_eq!(projectile_count(&mut app), 1, "one shot after the wind-up");
    let ranged = *app.world().get::<Ranged>(spitter).expect("ranged");
    assert!(!ranged.is_winding());
    assert!(ranged.cooldown_timer > 0.0);
}

#[test]
fn shot_is_cancelled_when_the_target_leaves_reach() {
    let mut app = ranged_app();
    let survivor = spawn_survivor(&mut app, Vec3::new(0.0, 0.9, 0.0));
    let spitter = spawn_spitter(&mut app, Vec3::new(0.0, 1.0, 8.0), survivor);

    app.update();
    app.world_mut()
        .get_mut::<Transform>(survivor)
        .expect("survivor transform")
        .translation = Vec3::new(0.0, 0.9, 40.0);

    for _ in 0..8 {
        app.update();
    }
    assert_eq!(projectile_count(&mut app), 0);
    let ranged = *app.world().get::<Ranged>(spitter).expect("ranged");
    assert!(ranged.cooldown_timer > 0.0, "the cooldown still starts");
}

#[test]
fn cooldown_gates_the_next_attack() {
    let mut app = ranged_app();
    let survivor = spawn_survivor(&mut app, Vec3::new(0.0, 0.9, 0.0));
    let spitter = spawn_spitter(&mut app, Vec3::new(0.0, 1.0, 8.0), survivor);

    for _ in 0..7 {
        app.update();
    }
    assert_eq!(projectile_count(&mut app), 1);

    app.update();
    let ranged = *app.world().get::<Ranged>(spitter).expect("ranged");
    assert!(!ranged.is_winding(), "cooldown must gate the next wind-up");
    assert_eq!(projectile_count(&mut app), 1);
}

#[test]
fn preferred_range_backs_away_holds_and_advances() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(0.1)))
        .insert_resource(config())
        .add_systems(Update, steer_enemies);

    let survivor = spawn_survivor(&mut app, Vec3::new(0.0, 0.9, 0.0));
    let spitter = spawn_spitter(&mut app, Vec3::new(0.0, 1.0, 8.0), survivor);
    app.world_mut()
        .entity_mut(spitter)
        .insert((FacingDirection(Vec3::NEG_Z), Velocity::zero()));

    app.update();
    let velocity = app.world().get::<Velocity>(spitter).expect("velocity").linear;
    assert!(velocity.z > 0.0, "inside preferred range must back away: {velocity:?}");

    app.world_mut()
        .get_mut::<Transform>(spitter)
        .expect("transform")
        .translation = Vec3::new(0.0, 1.0, 30.0);
    app.update();
    let velocity = app.world().get::<Velocity>(spitter).expect("velocity").linear;
    assert!(velocity.z < 0.0, "beyond preferred range must advance: {velocity:?}");

    app.world_mut()
        .get_mut::<Transform>(spitter)
        .expect("transform")
        .translation = Vec3::new(0.0, 1.0, 13.0);
    app.update();
    let velocity = app.world().get::<Velocity>(spitter).expect("velocity").linear;
    assert!(velocity.length() < 1e-3, "inside the hold band must not move: {velocity:?}");
}

#[test]
fn ballistic_launch_lands_near_the_target_distance() {
    let from = Vec3::new(0.0, 1.0, 0.0);
    let target = Vec3::new(10.0, 0.0, 0.0);
    let mut velocity = launch_velocity(from, target, 30.0, 35.0, 9.81).expect("solution");

    let mut position = from;
    let step = 0.001;
    let mut elapsed = 0.0;
    while position.y > 0.0 && elapsed < 10.0 {
        velocity.y -= 9.81 * step;
        position += velocity * step;
        elapsed += step;
    }
    assert!((position.x - 10.0).abs() < 0.2, "landed at x={}", position.x);
}

#[test]
fn projectile_collision_groups_ignore_combatants() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);

    let spec = spitter_spec(&config());
    let ranged = Ranged::from_spec(spec);
    let mut commands = app.world_mut().commands();
    let projectile = spawn_acid_projectile(&mut commands, Vec3::ZERO, Vec3::Z, &ranged);
    app.world_mut().flush();

    let groups = app
        .world()
        .get::<CollisionGroups>(projectile)
        .expect("collision groups");
    assert!(groups.filters.contains(GROUP_WORLD));
    assert!(!groups.filters.contains(GROUP_ENEMY), "must pass through enemies");
}
