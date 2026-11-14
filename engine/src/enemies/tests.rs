use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;

use crate::common::{Armor, DamageKind, Dying, EnemyKind};
use crate::config::{Behavior, ConfigHotReloadedMessage, EnemiesConfig, EnemyArchetypeData};
use crate::message::messages::{DamageMessage, EnemyKilledMessage};
use crate::world::{Health, PlayerConfig, Survivor};

use super::ai::state::{Enemy, FacingDirection};
use super::ai::system::{steer_enemies, sync_enemies_hot_reload};
use super::brain::system::update_brains;
use super::brain::{BrainState, EnemyBrain};
use super::damage::apply_enemy_damage;
use super::detonate::Fuse;
use super::melee::{Melee, enemy_melee};
use super::ranged::Ranged;
use super::spawn::{eligible_archetypes, pick_archetype};

fn config() -> EnemiesConfig {
    let config: EnemiesConfig = crate::assert_ron_config_parses!(EnemiesConfig);
    config.validate().expect("authored roster is valid");
    config
}

fn archetype(config: &EnemiesConfig, kind: EnemyKind) -> EnemyArchetypeData {
    config
        .archetype(kind)
        .expect("authored archetype")
        .clone()
}

fn with_time(app: &mut App, step_secs: f32) {
    app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
        Duration::from_secs_f32(step_secs),
    ));
}

fn minimal_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    with_time(&mut app, 1.0 / 60.0);
    app
}

fn spawn_enemy(app: &mut App, arch: &EnemyArchetypeData, pos: Vec3) -> Entity {
    app.world_mut()
        .spawn((
            Enemy::from_archetype(arch),
            EnemyBrain::default(),
            Health::new(arch.max_hp),
            FacingDirection(Vec3::NEG_Z),
            Transform::from_translation(pos),
            Velocity::zero(),
        ))
        .id()
}

#[derive(Resource, Default)]
struct DamageCount(usize);

#[derive(Resource, Default)]
struct KillCount(usize);

#[derive(Resource, Default)]
struct KillScrap(Vec<u32>);

fn count_damage(
    mut reader: MessageReader<DamageMessage>,
    mut count: ResMut<DamageCount>,
) {
    count.0 += reader.read().count();
}

fn count_kills(
    mut reader: MessageReader<EnemyKilledMessage>,
    mut count: ResMut<KillCount>,
    mut scrap: ResMut<KillScrap>,
) {
    for msg in reader.read() {
        count.0 += 1;
        scrap.0.push(msg.scrap);
    }
}

fn damage_app(config: EnemiesConfig) -> App {
    let mut app = minimal_app();
    app.insert_resource(config)
        .add_message::<DamageMessage>()
        .add_message::<EnemyKilledMessage>()
        .init_resource::<DamageCount>()
        .init_resource::<KillCount>()
        .init_resource::<KillScrap>()
        .add_systems(
            Update,
            (apply_enemy_damage, count_damage, count_kills).chain(),
        );
    app
}

fn hit(target: Entity, amount: f32, knockback: Vec3, kind: DamageKind, hit_dir: Vec3) -> DamageMessage {
    DamageMessage {
        target,
        amount,
        knockback,
        source: Entity::PLACEHOLDER,
        kind,
        hit_dir,
    }
}

#[test]
fn lethal_damage_marks_dying_once_and_carries_scrap() {
    let config = config();
    let arch = archetype(&config, EnemyKind::Tank);
    let mut app = damage_app(config);
    let enemy = spawn_enemy(&mut app, &arch, Vec3::new(0.0, 1.45, 0.0));

    app.world_mut()
        .write_message(hit(enemy, 20.0, Vec3::new(0.0, 0.0, -5.0), DamageKind::Ballistic, Vec3::Z));
    app.update();

    assert!(app.world().get::<Dying>(enemy).is_none());
    let health = app.world().get::<Health>(enemy).unwrap();
    assert!((health.current - 400.0).abs() < 0.01);
    assert!(app.world().resource::<DamageCount>().0 == 1);

    app.world_mut()
        .write_message(hit(enemy, 1000.0, Vec3::ZERO, DamageKind::Ballistic, Vec3::Z));
    app.update();

    assert!(app.world().get::<Dying>(enemy).is_some(), "must stay alive for death effects");
    assert_eq!(app.world().resource::<KillCount>().0, 1);
    assert_eq!(app.world().resource::<KillScrap>().0, vec![arch.scrap_value]);

    app.world_mut()
        .write_message(hit(enemy, 1000.0, Vec3::ZERO, DamageKind::Ballistic, Vec3::Z));
    app.update();
    assert_eq!(app.world().resource::<KillCount>().0, 1, "no second kill message");
}

#[test]
fn tank_armor_scales_front_back_and_bypass() {
    let config = config();
    let arch = archetype(&config, EnemyKind::Tank);
    let spec = arch.armor().expect("tank armor");
    let mut app = damage_app(config);

    let frontal = spawn_enemy(&mut app, &arch, Vec3::ZERO);
    let rear = spawn_enemy(&mut app, &arch, Vec3::ZERO);
    let bypass = spawn_enemy(&mut app, &arch, Vec3::ZERO);
    for entity in [frontal, rear, bypass] {
        app.world_mut().entity_mut(entity).insert(Armor {
            front_mult: spec.front_mult,
            back_mult: spec.back_mult,
            arc_deg: spec.arc_deg,
            bypass: spec.bypass.clone(),
        });
    }

    app.world_mut()
        .write_message(hit(frontal, 100.0, Vec3::ZERO, DamageKind::Ballistic, Vec3::Z));
    app.world_mut()
        .write_message(hit(rear, 100.0, Vec3::ZERO, DamageKind::Ballistic, Vec3::NEG_Z));
    app.world_mut()
        .write_message(hit(bypass, 100.0, Vec3::ZERO, DamageKind::Explosive, Vec3::Z));
    app.update();

    let damage = |entity| arch.max_hp - app.world().get::<Health>(entity).unwrap().current;
    assert!((damage(frontal) - 100.0 * spec.front_mult).abs() < 0.01);
    assert!((damage(rear) - 100.0 * spec.back_mult).abs() < 0.01);
    assert!((damage(bypass) - 100.0).abs() < 0.01, "bypass kind ignores armor");
}

#[test]
fn tank_survives_rifle_knockback_without_staggering() {
    let config = config();
    let tank = archetype(&config, EnemyKind::Tank);
    let rifle_knockback = 3.5;
    assert!(rifle_knockback / tank.mass < config.stagger_threshold);

    let mut app = damage_app(config);
    let enemy = spawn_enemy(&mut app, &tank, Vec3::ZERO);
    app.world_mut()
        .write_message(hit(enemy, 10.0, Vec3::X * rifle_knockback, DamageKind::Ballistic, Vec3::Z));
    app.update();

    let brain = app.world().get::<EnemyBrain>(enemy).unwrap();
    assert_ne!(brain.state, BrainState::Stagger);
}

#[test]
fn melee_windup_roots_then_resolves_in_reach() {
    let config = config();
    let tank = archetype(&config, EnemyKind::Tank);
    let spec = tank.melee().expect("tank melee");

    let mut app = minimal_app();
    app.init_resource::<PlayerConfig>()
        .insert_resource(config)
        .add_message::<DamageMessage>()
        .init_resource::<DamageCount>()
        .add_systems(Update, (enemy_melee, count_damage).chain());

    let survivor = app
        .world_mut()
        .spawn((Survivor, Transform::from_xyz(0.0, 0.9, 0.0)))
        .id();
    let enemy = app
        .world_mut()
        .spawn((
            Enemy::from_archetype(&tank),
            EnemyBrain {
                state: BrainState::Engage,
                target: Some(survivor),
                last_known: Some(Vec3::ZERO),
                ..default()
            },
            Melee::from_spec(spec),
            Transform::from_xyz(0.0, 1.45, 1.2),
        ))
        .id();

    app.update();
    let melee = app.world().get::<Melee>(enemy).unwrap();
    assert!(melee.is_winding(), "attack must telegraph before damage");
    assert_eq!(app.world().resource::<DamageCount>().0, 0);

    for _ in 0..30 {
        app.update();
    }
    assert_eq!(app.world().resource::<DamageCount>().0, 1);
    assert!(app.world().get::<Melee>(enemy).unwrap().cooldown_timer > 0.0);
}

#[test]
fn melee_misses_when_target_leaves_reach() {
    let config = config();
    let tank = archetype(&config, EnemyKind::Tank);
    let spec = tank.melee().expect("tank melee");

    let mut app = minimal_app();
    app.init_resource::<PlayerConfig>()
        .insert_resource(config)
        .add_message::<DamageMessage>()
        .init_resource::<DamageCount>()
        .add_systems(Update, (enemy_melee, count_damage).chain());

    let survivor = app
        .world_mut()
        .spawn((Survivor, Transform::from_xyz(0.0, 0.9, 0.0)))
        .id();
    app.world_mut().spawn((
        Enemy::from_archetype(&tank),
        EnemyBrain {
            state: BrainState::Engage,
            target: Some(survivor),
            last_known: Some(Vec3::ZERO),
            ..default()
        },
        Melee::from_spec(spec),
        Transform::from_xyz(0.0, 1.45, 1.2),
    ));

    app.update();
    app.world_mut().get_mut::<Transform>(survivor).unwrap().translation = Vec3::new(0.0, 0.9, 12.0);
    for _ in 0..30 {
        app.update();
    }
    assert_eq!(app.world().resource::<DamageCount>().0, 0);
}

#[test]
fn brain_engages_degaggros_and_returns_to_idle() {
    let config = config();
    let walker = archetype(&config, EnemyKind::Walker);

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    with_time(&mut app, 0.1);
    app.insert_resource(config).add_systems(Update, update_brains);

    let survivor = app
        .world_mut()
        .spawn((Survivor, Transform::from_xyz(0.0, 0.9, 0.0)))
        .id();
    let enemy = spawn_enemy(&mut app, &walker, Vec3::new(0.0, 0.9, 5.0));

    app.update();
    let brain = app.world().get::<EnemyBrain>(enemy).unwrap();
    assert_eq!(brain.state, BrainState::Engage);
    assert_eq!(brain.target, Some(survivor));

    app.world_mut().get_mut::<Transform>(survivor).unwrap().translation =
        Vec3::new(0.0, 0.9, 100.0);
    for _ in 0..30 {
        app.update();
    }
    assert_eq!(app.world().get::<EnemyBrain>(enemy).unwrap().state, BrainState::Alert);

    for _ in 0..50 {
        app.update();
    }
    let brain = app.world().get::<EnemyBrain>(enemy).unwrap();
    assert_eq!(brain.state, BrainState::Idle);
    assert!(brain.target.is_none());
}

#[test]
fn separation_pushes_overlapping_enemies_apart() {
    let config = config();
    let walker = archetype(&config, EnemyKind::Walker);

    let mut app = minimal_app();
    app.insert_resource(config).add_systems(Update, steer_enemies);

    let left = app
        .world_mut()
        .spawn((
            Enemy::from_archetype(&walker),
            EnemyBrain {
                state: BrainState::Engage,
                ..default()
            },
            FacingDirection(Vec3::NEG_Z),
            Transform::from_xyz(0.0, 0.9, 0.0),
            Velocity::zero(),
        ))
        .id();
    let right = app
        .world_mut()
        .spawn((
            Enemy::from_archetype(&walker),
            EnemyBrain {
                state: BrainState::Engage,
                ..default()
            },
            FacingDirection(Vec3::NEG_Z),
            Transform::from_xyz(0.2, 0.9, 0.0),
            Velocity::zero(),
        ))
        .id();

    app.update();

    let left_x = app.world().get::<Velocity>(left).unwrap().linear.x;
    let right_x = app.world().get::<Velocity>(right).unwrap().linear.x;
    assert!(left_x < 0.0, "left enemy must be pushed away: {left_x}");
    assert!(right_x > 0.0, "right enemy must be pushed away: {right_x}");
}

#[test]
fn spawn_eligibility_respects_tier_and_caps() {
    let config = config();
    let counts = vec![
        (EnemyKind::Walker, 0),
        (EnemyKind::Runner, 0),
        (EnemyKind::Tank, 0),
    ];

    let early = eligible_archetypes(&config, 0.0, &counts, 0, 80);
    assert!(early.iter().all(|arch| arch.kind != EnemyKind::Tank));
    assert!(!early.is_empty());

    let late = eligible_archetypes(&config, 120.0, &counts, 0, 80);
    assert!(late.iter().any(|arch| arch.kind == EnemyKind::Tank));

    let capped = vec![
        (EnemyKind::Walker, 60),
        (EnemyKind::Runner, 0),
        (EnemyKind::Tank, 0),
    ];
    let eligible = eligible_archetypes(&config, 200.0, &capped, 60, 80);
    assert!(eligible.iter().all(|arch| arch.kind != EnemyKind::Walker));
    assert!(eligible.iter().any(|arch| arch.kind == EnemyKind::Runner));

    assert!(eligible_archetypes(&config, 200.0, &counts, 80, 80).is_empty());
}

#[test]
fn weighted_pick_matches_authored_weights() {
    let config = config();
    let counts = vec![
        (EnemyKind::Walker, 0),
        (EnemyKind::Runner, 0),
        (EnemyKind::Tank, 0),
    ];
    let eligible = eligible_archetypes(&config, 0.0, &counts, 0, 80);

    assert_eq!(
        pick_archetype(&eligible, 0.0).map(|arch| arch.kind),
        Some(EnemyKind::Walker)
    );
    assert_eq!(
        pick_archetype(&eligible, 0.69).map(|arch| arch.kind),
        Some(EnemyKind::Walker)
    );
    assert_eq!(
        pick_archetype(&eligible, 0.71).map(|arch| arch.kind),
        Some(EnemyKind::Runner)
    );
}

#[test]
fn hot_reload_updates_live_armor_and_melee_values() {
    let config = config();
    let tank = archetype(&config, EnemyKind::Tank);
    let armor_spec = tank.armor().expect("tank armor");
    let melee_spec = tank.melee().expect("tank melee");

    let mut app = minimal_app();
    app.insert_resource(config.clone())
        .add_message::<ConfigHotReloadedMessage>()
        .add_systems(Update, sync_enemies_hot_reload);

    let entity = spawn_enemy(&mut app, &tank, Vec3::ZERO);
    app.world_mut().entity_mut(entity).insert((
        Armor {
            front_mult: armor_spec.front_mult,
            back_mult: armor_spec.back_mult,
            arc_deg: armor_spec.arc_deg,
            bypass: armor_spec.bypass.clone(),
        },
        Melee::from_spec(melee_spec),
    ));
    app.update();

    {
        let mut cfg = app.world_mut().resource_mut::<EnemiesConfig>();
        let arch = cfg
            .archetypes
            .iter_mut()
            .find(|arch| arch.kind == EnemyKind::Tank)
            .expect("tank archetype");
        for behavior in &mut arch.behaviors {
            match behavior {
                Behavior::Armor { front_mult, .. } => *front_mult = 0.1,
                Behavior::Melee { damage, .. } => *damage = 99.0,
                _ => {}
            }
        }
    }
    app.world_mut().write_message(ConfigHotReloadedMessage);
    app.update();

    let armor = app.world().get::<Armor>(entity).expect("armor retained");
    assert!((armor.front_mult - 0.1).abs() < 1e-6);
    let melee = app.world().get::<Melee>(entity).expect("melee retained");
    assert!((melee.damage - 99.0).abs() < 1e-6);
}

#[test]
fn hot_reload_updates_live_ranged_and_detonation_values() {
    let config = config();
    let spitter = archetype(&config, EnemyKind::Spitter);
    let exploder = archetype(&config, EnemyKind::Exploder);
    let spitter_spec = spitter.ranged().expect("spitter ranged");
    let detonate_spec = exploder.detonate().expect("exploder detonate");

    let mut app = minimal_app();
    app.insert_resource(config.clone())
        .add_message::<ConfigHotReloadedMessage>()
        .add_systems(Update, sync_enemies_hot_reload);

    let spitter_entity = spawn_enemy(&mut app, &spitter, Vec3::ZERO);
    app.world_mut()
        .entity_mut(spitter_entity)
        .insert(Ranged::from_spec(spitter_spec));
    let exploder_entity = spawn_enemy(&mut app, &exploder, Vec3::ZERO);
    app.world_mut()
        .entity_mut(exploder_entity)
        .insert(Fuse::from_spec(detonate_spec));
    app.update();

    {
        let mut cfg = app.world_mut().resource_mut::<EnemiesConfig>();
        for arch in &mut cfg.archetypes {
            match arch.kind {
                EnemyKind::Spitter => {
                    for behavior in &mut arch.behaviors {
                        if let Behavior::Ranged {
                            damage, pool_radius, ..
                        } = behavior
                        {
                            *damage = 21.0;
                            *pool_radius = 3.5;
                        }
                    }
                }
                EnemyKind::Exploder => {
                    for behavior in &mut arch.behaviors {
                        if let Behavior::Detonate { fuse, radius, .. } = behavior {
                            *fuse = 1.4;
                            *radius = 6.0;
                        }
                    }
                }
                _ => {}
            }
        }
    }
    app.world_mut().write_message(ConfigHotReloadedMessage);
    app.update();

    let ranged = app.world().get::<Ranged>(spitter_entity).expect("ranged retained");
    assert!((ranged.damage - 21.0).abs() < 1e-6);
    assert!((ranged.pool_radius - 3.5).abs() < 1e-6);

    let fuse = app.world().get::<Fuse>(exploder_entity).expect("fuse retained");
    assert!((fuse.fuse - 1.4).abs() < 1e-6);
    assert!((fuse.radius - 6.0).abs() < 1e-6);
    assert!(!fuse.armed, "value reload must not arm a fuse");
}

#[test]
fn hot_reload_spawn_policy_applies_to_next_roll() {
    let mut config = config();
    let counts = vec![
        (EnemyKind::Walker, 0),
        (EnemyKind::Runner, 0),
        (EnemyKind::Tank, 0),
    ];

    assert!(
        eligible_archetypes(&config, 0.0, &counts, 0, 80)
            .iter()
            .all(|arch| arch.kind != EnemyKind::Tank)
    );

    for arch in &mut config.archetypes {
        if arch.kind == EnemyKind::Tank {
            arch.spawn.unlock_secs = 0.0;
            arch.spawn.weight = 1.0;
        }
    }

    let eligible = eligible_archetypes(&config, 0.0, &counts, 0, 80);
    assert!(eligible.iter().any(|arch| arch.kind == EnemyKind::Tank));
    assert_eq!(
        pick_archetype(&eligible, 0.999).map(|arch| arch.kind),
        Some(EnemyKind::Tank)
    );

    let capped = vec![
        (EnemyKind::Walker, 0),
        (EnemyKind::Runner, 0),
        (EnemyKind::Tank, 6),
    ];
    assert!(
        eligible_archetypes(&config, 0.0, &capped, 6, 80)
            .iter()
            .all(|arch| arch.kind != EnemyKind::Tank)
    );
}
