use std::time::Duration;

use bevy_rapier3d::{dynamics::Velocity, plugin::{NoUserData, RapierPhysicsPlugin, TimestepMode}};
use crate::config::ConfigHotReloadedMessage;
use crate::message::messages::{DamageMessage, EnemyKilledMessage};
use crate::common::EnemyKind;
use crate::world::{ArenaConfig, Health, PlayerConfig, Survivor};

use super::*;

#[test]
fn enemy_damage_applies_knockback_and_emits_kill_message() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            Duration::from_secs_f32(1.0 / 60.0),
        ))
        .add_message::<DamageMessage>()
        .add_message::<EnemyKilledMessage>()
        .add_systems(Update, apply_enemy_damage);

    let runner_arch = EnemiesConfig::default().runner;
    let enemy = app
        .world_mut()
        .spawn((
            Enemy::from_archetype(EnemyKind::Runner, runner_arch),
            Health::new(runner_arch.max_hp),
            Transform::from_xyz(3.0, 0.9, -4.0),
        ))
        .id();

    app.world_mut().write_message(DamageMessage {
        target: enemy,
        amount: 15.0,
        knockback: Vec3::new(0.0, 0.0, -5.0),
        source: Entity::PLACEHOLDER,
    });
    app.update();

    let hp = app.world().get::<Health>(enemy).unwrap().current;
    assert!((hp - 20.0).abs() < 0.01);
    assert!(app.world().get::<Enemy>(enemy).unwrap().knockback_vel.z < -1.0);

    app.world_mut().write_message(DamageMessage {
        target: enemy,
        amount: 25.0,
        knockback: Vec3::ZERO,
        source: Entity::PLACEHOLDER,
    });
    app.update();

    assert!(app.world().get_entity(enemy).is_err());
    let msgs = app.world().resource::<Messages<EnemyKilledMessage>>();
    let mut cursor = msgs.get_cursor();
    let kills: Vec<_> = cursor.read(msgs).copied().collect();
    assert_eq!(kills.len(), 1);
    assert_eq!(kills[0].kind, EnemyKind::Runner);
}

#[test]
fn enemy_chases_and_deals_contact_damage_on_cooldown() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<PlayerConfig>()
        .init_resource::<EnemiesConfig>()
        .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            Duration::from_secs_f32(1.0 / 60.0),
        ))
        .add_message::<DamageMessage>()
        .add_systems(Update, enemy_chase_and_attack);

    let walker_arch = EnemiesConfig::default().walker;
    let survivor = app
        .world_mut()
        .spawn((Survivor, Transform::from_xyz(0.0, 0.9, 0.0)))
        .id();
    let zombie = app
        .world_mut()
        .spawn((
            Enemy::from_archetype(EnemyKind::Walker, walker_arch),
            FacingDirection(Vec3::NEG_Z),
            Velocity::zero(),
            Transform::from_xyz(0.0, 0.9, 0.8),
        ))
        .id();

    app.update();

    let vel = app.world().get::<Velocity>(zombie).unwrap().linear;
    assert!(vel.z < -1.0);

    let messages = app.world().resource::<Messages<DamageMessage>>();
    let mut reader = messages.get_cursor();
    let hits: Vec<_> = reader.read(messages).copied().collect();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].target, survivor);
    assert_eq!(hits[0].amount, walker_arch.attack_damage);
}

#[test]
fn walker_and_runner_move_with_rapier_when_player_stationary() {
    use bevy::ecs::system::RunSystemOnce;
    use crate::world::arena::spawn_arena_physics;
    use crate::world::player::spawn_player;

    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::asset::AssetPlugin::default(),
        bevy::mesh::MeshPlugin,
        bevy::world_serialization::WorldSerializationPlugin,
        bevy::transform::TransformPlugin,
        RapierPhysicsPlugin::<NoUserData>::default(),
    ))
    .insert_resource(TimestepMode::Interpolated {
        dt: 1.0 / 60.0,
        time_scale: 1.0,
        substeps: 1,
    })
    .init_resource::<ArenaConfig>()
    .init_resource::<PlayerConfig>()
    .init_resource::<EnemiesConfig>()
    .init_resource::<crate::message::resources::UpgradeModifiers>()
    .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        Duration::from_secs_f32(1.0 / 60.0),
    ))
    .add_message::<DamageMessage>()
    .add_message::<ConfigHotReloadedMessage>()
    .add_systems(Startup, (spawn_arena_physics, spawn_player))
    .add_systems(
        Update,
        (
            crate::world::player::player_movement_and_aim,
            enemy_chase_and_attack,
        )
            .chain(),
    );

    app.update();

    let cfg = app.world().resource::<EnemiesConfig>().clone();
    let (walker, runner) = app
        .world_mut()
        .run_system_once(move |mut commands: Commands| {
            let w = spawn_enemy_entity(
                &mut commands,
                EnemyKind::Walker,
                cfg.walker,
                Vec3::new(10.0, 0.9, 0.0),
            );
            let r = spawn_enemy_entity(
                &mut commands,
                EnemyKind::Runner,
                cfg.runner,
                Vec3::new(-10.0, 0.9, 0.0),
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

