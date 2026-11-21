use std::f32::consts::{FRAC_PI_2, PI};
use std::time::Duration;

use bevy_rapier2d::prelude::{ExternalForce, Velocity};

use super::*;
use crate::config::{GameConfig, GameSettings};
use crate::content::block::feature::component::Feature;
use crate::content::block::feature::power::Booster;
use crate::content::block::feature::thruster::Thruster;
use crate::content::block::feature::turret::TurretComponent;
use crate::content::block::{
    BakedShape, BlockDef, BlockOwner, BlockPlacement, BlockRegistry, Disabled, Rot90, ShapeSpec,
    ShootPattern, ShootingProjectile, SingleShootPattern, TargetingStrategy,
};
use crate::content::body::BlockGrid;
use crate::content::damage::DamageEvent;
use crate::content::ship::{Ship, ShipStats, Throttle, update_ship_steering};
use crate::content::team::Team;
use crate::scene::game::skirmish::resolve_skill;
use crate::resource::ControlledShip;

fn make_registry() -> BlockRegistry {
    let mut reg = BlockRegistry::default();
    let square = BakedShape::try_from(&ShapeSpec::Square(1)).unwrap();
    let thruster: Thruster = ron::from_str("(force: 200.0, torque: 100.0)").unwrap();

    reg.register(
        "core",
        BlockDef {
            name: "core".into(),
            block_type: "core".into(),
            block_group: "core".into(),
            mass: 50.0,
            health: 5000.0,
            shape: square.clone(),
            cost: 10,
            features: vec![Feature::Thruster(thruster)],
            drops: vec![],
        },
    );
    reg.register(
        "hull",
        BlockDef {
            name: "hull".into(),
            block_type: "hull".into(),
            block_group: "base".into(),
            mass: 10.0,
            health: 1000.0,
            shape: square.clone(),
            cost: 5,
            features: vec![],
            drops: vec![],
        },
    );
    reg.register(
        "rail",
        BlockDef {
            name: "rail".into(),
            block_type: "rail".into(),
            block_group: "turret".into(),
            mass: 20.0,
            health: 2000.0,
            shape: square.clone(),
            cost: 50,
            features: vec![Feature::Turret(TurretComponent {
                strategy: TargetingStrategy::Closest,
                shooting_pattern: ShootPattern::Single(SingleShootPattern {
                    fire_rate: 5.0,
                    projectile: ShootingProjectile::Bullet {
                        speed: 10000.0,
                        damage: 2500.0,
                        lifetime: 10.0,
                        piercing: 1,
                        width: 4.0,
                        length: 40.0,
                    },
                }),
                turn_speed: 0.0,
                aim_tolerance: 0.035,
                range: 15000.0,
                barrels: vec![Vec2::ZERO],
                image: Handle::default(),
                offset: Vec2::ZERO,
                animated: false,
                ammo: None,
                power: None,
            })],
            drops: vec![],
        },
    );
    reg.register(
        "gun",
        BlockDef {
            name: "gun".into(),
            block_type: "gun".into(),
            block_group: "turret".into(),
            mass: 10.0,
            health: 1000.0,
            shape: square.clone(),
            cost: 20,
            features: vec![Feature::Turret(TurretComponent {
                strategy: TargetingStrategy::Closest,
                shooting_pattern: ShootPattern::Single(SingleShootPattern {
                    fire_rate: 1.0,
                    projectile: ShootingProjectile::Laser {
                        damage: 40.0,
                        width: 2.0,
                    },
                }),
                turn_speed: 3.0,
                aim_tolerance: PI / 16.0,
                range: 2000.0,
                barrels: vec![Vec2::ZERO],
                image: Handle::default(),
                offset: Vec2::ZERO,
                animated: false,
                ammo: None,
                power: None,
            })],
            drops: vec![],
        },
    );
    reg.register(
        "booster",
        BlockDef {
            name: "booster".into(),
            block_type: "booster".into(),
            block_group: "base".into(),
            mass: 10.0,
            health: 1000.0,
            shape: square,
            cost: 20,
            features: vec![Feature::Booster(Booster {
                radius: 256.0,
                boost: 1.5,
                rate: 8.0,
            })],
            drops: vec![],
        },
    );
    reg
}

#[test]
fn capability_updates_on_fixed_weapon_loss_and_detects_support_hulls() {
    let reg = make_registry();
    let ai_cfg = AiConfig::default();

    let mut sniper_grid = BlockGrid::new();
    let rail = BlockPlacement::new(IVec2::Y, "rail", Rot90::R90);
    let rail_id = rail.id;
    sniper_grid.rebuild_from_layout(
        &[
            BlockPlacement::new(IVec2::ZERO, "core", Rot90::R0),
            rail,
            BlockPlacement::new(IVec2::X, "gun", Rot90::R0),
        ],
        &reg,
    );
    let build = reg.build_grid_result(&sniper_grid).unwrap();
    let cap_before = compute_capability(
        &sniper_grid,
        &build.ship,
        &build.stats,
        Doctrine::Auto,
        &reg,
        &ai_cfg,
        None,
    );
    assert_eq!(cap_before.role, Role::Sniper);
    assert!((cap_before.fixed_mount.unwrap().mount_angle - FRAC_PI_2).abs() < 1e-4);
    assert_eq!(cap_before.max_radar_range, 15000.0);

    sniper_grid.remove_block(rail_id);
    let build_after = reg.build_grid_result(&sniper_grid).unwrap();
    let cap_after = compute_capability(
        &sniper_grid,
        &build_after.ship,
        &build_after.stats,
        Doctrine::Auto,
        &reg,
        &ai_cfg,
        None,
    );
    assert!(cap_after.fixed_mount.is_none());
    assert_eq!(cap_after.role, Role::Brawler);
    assert_eq!(cap_after.max_radar_range, ai_cfg.base_core_radar_range);

    let mut support_grid = BlockGrid::new();
    support_grid.rebuild_from_layout(
        &[
            BlockPlacement::new(IVec2::ZERO, "core", Rot90::R0),
            BlockPlacement::new(IVec2::X, "booster", Rot90::R0),
        ],
        &reg,
    );
    let s_build = reg.build_grid_result(&support_grid).unwrap();
    let cap_support = compute_capability(
        &support_grid,
        &s_build.ship,
        &s_build.stats,
        Doctrine::Auto,
        &reg,
        &ai_cfg,
        None,
    );
    assert_eq!(cap_support.role, Role::Support);
}

#[test]
fn team_intel_shares_contacts_memory_and_damage_alerts() {
    let mut app = App::new();
    app.init_resource::<GameConfig>()
        .init_resource::<TacticalBoard>()
        .init_resource::<Time>()
        .add_message::<DamageEvent>()
        .add_systems(Update, update_sensors_and_intel);

    let spotter = app
        .world_mut()
        .spawn((
            Team::new(1),
            Ship::default(),
            ShipStats::default(),
            Capability {
                max_radar_range: 1000.0,
                ..default()
            },
            Skill::new(1), // Lowest skill STILL shares team intel!
            Sensor::default(),
            GlobalTransform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();

    let wingman = app
        .world_mut()
        .spawn((
            Team::new(1),
            Ship::default(),
            ShipStats::default(),
            Capability {
                max_radar_range: 500.0,
                ..default()
            },
            Skill::new(1),
            Sensor::default(),
            GlobalTransform::from_xyz(9000.0, 0.0, 0.0),
        ))
        .id();

    let player = app
        .world_mut()
        .spawn((
            Team::new(0),
            Ship::default(),
            ShipStats {
                max_health: 1000.0,
                health: 1000.0,
                ..default()
            },
            Capability {
                max_radar_range: 500.0,
                offensive_dps: 100.0,
                ..default()
            },
            GlobalTransform::from_xyz(600.0, 0.0, 0.0),
        ))
        .id();

    app.update();

    let board = app.world().resource::<TacticalBoard>();
    let team1 = board.teams.get(&1).expect("team 1 intel exists");
    assert!(team1.intel.contacts.contains_key(&player));
    assert!(app.world().get::<Sensor>(spotter).unwrap().local_contacts.contains(&player));
    assert!(app.world().get::<Sensor>(wingman).unwrap().local_contacts.is_empty());

    // Despawn spotter -> player persists in TeamIntel.last_known
    app.world_mut().entity_mut(spotter).despawn();
    app.update();
    let team1 = app.world().resource::<TacticalBoard>().teams.get(&1).unwrap();
    assert!(!team1.intel.contacts.contains_key(&player));
    assert!(team1.intel.last_known.contains_key(&player));

    // Unseen attacker damages wingman -> TeamAlert posted
    let victim_block = app.world_mut().spawn(BlockOwner { body: wingman }).id();
    let sniper_shot = app
        .world_mut()
        .spawn(GlobalTransform::from_xyz(14000.0, 0.0, 0.0))
        .id();
    app.world_mut().write_message(DamageEvent {
        attacker: sniper_shot,
        target: victim_block,
        damage: 50.0,
    });
    app.update();
    let team1 = app.world().resource::<TacticalBoard>().teams.get(&1).unwrap();
    assert_eq!(team1.intel.alerts.len(), 1);
}

#[test]
fn utility_hysteresis_retreat_and_booster_power_shedding() {
    let cfg = AiConfig::default();
    let enemy = Entity::from_bits(99);
    let self_entity = Entity::from_bits(1);
    let mut plan = TeamPlan::default();
    plan.intel.contacts.insert(
        enemy,
        SharedContact {
            entity: enemy,
            position: Vec2::new(1000.0, 0.0),
            velocity: Vec2::ZERO,
            estimated_threat: 100.0,
            has_fixed_gun: false,
            heading: Vec2::X,
        },
    );
    let cap = Capability {
        role: Role::Brawler,
        offensive_dps: 100.0,
        optimal_range: 1500.0,
        max_range: 2000.0,
        has_rotating_turrets: true,
        can_move: true,
        booster_count: 1,
        ..default()
    };

    // Disarmed / critical HP transitions to Retreat
    let ctx_retreat = EvalContext {
        entity: self_entity,
        position: Vec2::ZERO,
        spawn_position: Vec2::ZERO,
        skill: 5,
        cap: &cap,
        readiness: &Readiness {
            hp_ratio: 0.1,
            ..default()
        },
        plan: &plan,
        cfg: &cfg,
        current: &Tactic::Search {
            waypoint: Vec2::ZERO,
        },
        nearest_contact: plan.intel.contacts.get(&enemy).copied(),
        preferred_contact: plan.intel.contacts.get(&enemy).copied(),
        vip_position: None,
    };
    assert!(matches!(select_tactic(&ctx_retreat), Tactic::Retreat { .. }));

    // Booster power-shedding test
    let mut app = App::new();
    app.init_resource::<GameConfig>()
        .init_resource::<TacticalBoard>()
        .init_resource::<ControlledShip>()
        .insert_resource(Time::<()>::default())
        .add_systems(Update, evaluate_utility_and_power);

    let ship = app
        .world_mut()
        .spawn((
            Team::new(1),
            Ship::default(),
            Skill::new(5),
            cap,
            Readiness {
                power_supply: 0.4,
                battery_ratio: 0.1,
                ..default()
            },
            Brain::new(Vec2::ZERO, 10.0),
            GlobalTransform::default(),
        ))
        .id();
    let booster = app
        .world_mut()
        .spawn((
            BlockOwner { body: ship },
            Booster {
                radius: 256.0,
                boost: 1.5,
                rate: 8.0,
            },
        ))
        .id();

    app.world_mut()
        .resource_mut::<Time<()>>()
        .advance_by(Duration::from_millis(200));
    app.update();
    assert!(app.world().get::<Disabled>(booster).is_some());

    // Recover battery -> booster re-enabled
    app.world_mut().get_mut::<Readiness>(ship).unwrap().battery_ratio = 0.8;
    app.world_mut()
        .resource_mut::<Time<()>>()
        .advance_by(Duration::from_millis(200));
    app.update();
    assert!(app.world().get::<Disabled>(booster).is_none());
}

#[test]
fn steering_respects_skill_omnidirectional_thrust_and_skill_precedence() {
    let ai_cfg = AiConfig::default();
    let setting = GameSettings {
        ai_level: 3,
        ..default()
    };
    assert_eq!(resolve_skill(Some(9), Team::new(1), Some(4), setting.ai_level, &ai_cfg).level(), 9);
    assert_eq!(resolve_skill(None, Team::new(1), Some(6), setting.ai_level, &ai_cfg).level(), 6);
    assert_eq!(resolve_skill(None, Team::new(1), None, setting.ai_level, &ai_cfg).level(), 3);
    assert_eq!(resolve_skill(None, Team::new(0), Some(2), setting.ai_level, &ai_cfg).level(), 8);

    let mut app = App::new();
    app.init_resource::<GameConfig>()
        .init_resource::<ControlledShip>()
        .insert_resource(GameConfig::default())
        .add_systems(Update, update_ship_steering);

    let high = app
        .world_mut()
        .spawn((
            Steering {
                desired_heading: Some(0.0), // Facing +Y
                thrust_world: Vec2::X,      // Lateral strafe +X
                throttle_cap: 1.0,
            },
            Skill::new(8),
            Transform::default(),
            ExternalForce::default(),
            Velocity::zero(),
            ShipStats {
                thrust_power: 100.0,
                rotate_speed: 50.0,
                ..default()
            },
            Throttle::default(),
        ))
        .id();

    let low = app
        .world_mut()
        .spawn((
            Steering {
                desired_heading: Some(0.0), // Facing +Y
                thrust_world: Vec2::X,      // Lateral intent should NOT strafe at low skill
                throttle_cap: 1.0,
            },
            Skill::new(2),
            Transform::default(),
            ExternalForce::default(),
            Velocity::zero(),
            ShipStats {
                thrust_power: 100.0,
                rotate_speed: 50.0,
                ..default()
            },
            Throttle::default(),
        ))
        .id();

    app.update();
    assert!(app.world().get::<ExternalForce>(high).unwrap().force.x > 90.0);
    assert!(app.world().get::<ExternalForce>(low).unwrap().force.length() < 1e-3);
}

#[test]
fn team_intel_alerts_and_last_known_respect_config_bounds() {
    let mut cfg = AiConfig::default();
    cfg.bounds.max_team_alerts = 2;
    cfg.bounds.alert_merge_radius = 50.0;

    let mut app = App::new();
    app.insert_resource(GameConfig { ai: cfg, ..default() })
        .init_resource::<TacticalBoard>()
        .init_resource::<Time>()
        .add_message::<DamageEvent>()
        .add_systems(Update, update_sensors_and_intel);

    let victim = app
        .world_mut()
        .spawn((
            Team::new(1),
            Ship::default(),
            ShipStats::default(),
            Capability {
                max_radar_range: 100.0,
                ..default()
            },
            Skill::new(5),
            Sensor::default(),
            GlobalTransform::from_xyz(0.0, 0.0, 0.0),
        ))
        .id();
    let block = app.world_mut().spawn(BlockOwner { body: victim }).id();

    for i in 1..=4 {
        let shot = app
            .world_mut()
            .spawn(GlobalTransform::from_xyz(i as f32 * 1000.0, 0.0, 0.0))
            .id();
        app.world_mut().write_message(DamageEvent {
            attacker: shot,
            target: block,
            damage: 10.0,
        });
    }
    app.update();

    let team1 = app.world().resource::<TacticalBoard>().teams.get(&1).unwrap();
    assert_eq!(team1.intel.alerts.len(), 2);
}
