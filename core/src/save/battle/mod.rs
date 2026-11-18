pub mod component;
pub mod lib;
pub mod system;

pub use component::*;
pub use lib::*;
pub use system::*;

use bevy::prelude::*;

use crate::resource::{BattleMode, PendingLoad, SceneState};
use crate::simulation::SimulationSet;

pub struct BattleSavePlugin;

impl Plugin for BattleSavePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SaveWorldEvent>()
            .add_message::<LoadWorldEvent>()
            .init_resource::<SaveResult>()
            .init_resource::<PendingLoad>()
            .init_resource::<BattleMode>()
            .init_resource::<PendingExitAction>()
            .init_resource::<BattleSaveState>()
            .add_systems(PreStartup, init_battle_save_state)
            .add_systems(
                Update,
                (
                    handle_save_world_event
                        .run_if(on_message::<SaveWorldEvent>)
                        .before(flush_pending_exit_action),
                    flush_pending_exit_action,
                    handle_load_request.run_if(on_message::<LoadWorldEvent>),
                    report_save_result,
                )
                    .in_set(SimulationSet::Present),
            )
            .add_systems(
                Update,
                begin_loaded_game
                    .run_if(in_state(SceneState::Loading))
                    .in_set(SimulationSet::Present),
            )
            .add_systems(OnEnter(SceneState::Game), spawn_loaded_world);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_rapier2d::prelude::Velocity;
    use uuid::Uuid;
    use crate::config::{GameConfig, GameConfigValue};
    use crate::content::block::feature::component::Feature;
    use crate::content::block::feature::power::{Battery, Charge};
    use crate::content::block::feature::turret::{
        TurretBarrel, TurretCharge, TurretComponent, TurretPower,
    };
    use crate::content::block::{
        BakedShape, BlockDef, BlockPlacement, BlockRegistry, ItemDrop, Loot, Rot90, ShapeSpec,
        ShootPattern, ShootingProjectile, SingleShootPattern, SingleShootPatternState,
        TargetingStrategy, spawn_block_entity,
    };
    use crate::content::body::Debris;
    use crate::content::lifetime::Lifetime;
    use crate::content::ship::ai::{AiBehavior, AiRuntime};
    use crate::content::ship::stat::ShipStats;
    use crate::content::body::BlockGrid;
    use crate::content::ship::ShipDesign;
    use crate::content::team::Team;
    use crate::resource::{CameraTarget, ControlMode, ControlledShip};
    use crate::save::campaign::campaign_save_path;
    use crate::save::document::parse_document;

    fn registry() -> BlockRegistry {
        let mut registry = BlockRegistry::default();
        registry.register(
            "core",
            BlockDef {
                name: "core".to_string(),
                block_type: "core".to_string(),
                block_group: "core".to_string(),
                mass: 50.0,
                health: 5000.0,
                shape: BakedShape::try_from(&ShapeSpec::Square(1)).expect("test shape bakes"),
                cost: 1,
                features: vec![Feature::Storage(100), Feature::Battery(Battery { capacity: 100.0 })],
                drops: vec![Loot::Item(ItemDrop {
                    item_type: "steel".to_string(),
                    amount: 10,
                })],
            },
        );
        registry.register(
            "gun",
            BlockDef {
                name: "gun".to_string(),
                block_type: "gun".to_string(),
                block_group: "weapon".to_string(),
                mass: 10.0,
                health: 1000.0,
                shape: BakedShape::try_from(&ShapeSpec::Square(1)).expect("test shape bakes"),
                cost: 1,
                features: vec![Feature::Turret(TurretComponent {
                    strategy: TargetingStrategy::Closest,
                    shooting_pattern: ShootPattern::Single(SingleShootPattern {
                        fire_rate: 1.0,
                        projectile: ShootingProjectile::Laser {
                            damage: 10.0,
                            width: 2.0,
                        },
                    }),
                    turn_speed: 2.0,
                    aim_tolerance: std::f32::consts::FRAC_PI_8 * 0.5,
                    range: 500.0,
                    barrels: vec![Vec2::ZERO],
                    image: Handle::default(),
                    offset: Vec2::ZERO,
                    z: 1.0,
                    animated: false,
                    ammo: None,
                    power: Some(TurretPower::Charge {
                        capacity: 20.0,
                        charge_rate: 5.0,
                    }),
                })],
                drops: Vec::new(),
            },
        );
        registry
    }

    #[test]
    fn world_save_v6_roundtrip_rejects_v5_and_v4() {
        let save = WorldSave {
            version: SAVE_SCHEMA_VERSION,
            saved_at_unix_ms: 1_700_000_000_000,
            battle: BattleSave {
                mission: "quick-battle".to_string(),
                elapsed_secs: 42.5,
                outcome: Some("Victory".to_string()),
            },
            camera: CameraSave {
                x: 12.0,
                y: -4.0,
                zoom: 3.0,
                following: None,
            },
            control: ControlSave {
                controlling: None,
                pilot_mode: false,
            },
            bodies: vec![BodySnapshot::Debris(DebrisSnapshot {
                team: 1,
                transform: TransformSave {
                    x: 5.0,
                    y: -2.0,
                    rotation: 0.25,
                },
                velocity: VelocitySave {
                    x: 1.0,
                    y: 2.0,
                    angvel: 0.5,
                },
                remaining_secs: 42.0,
                blocks: Vec::new(),
            })],
            drops: Vec::new(),
            rng: RngSave { seed: 1234 },
        };
        let text = ron::ser::to_string_pretty(&save, ron::ser::PrettyConfig::default()).unwrap();
        let back: WorldSave = parse_document(&text).unwrap();
        assert_eq!(back, save);

        let v5 = text.replace(&format!("version: {SAVE_SCHEMA_VERSION}"), "version: 5");
        let error = parse_document::<WorldSave>(&v5).expect_err("v5 is rejected");
        assert!(
            error.to_string().contains("expected version 6, found 5"),
            "{error}"
        );

        let v4 = text.replace(&format!("version: {SAVE_SCHEMA_VERSION}"), "version: 4");
        assert!(parse_document::<WorldSave>(&v4).is_err());

        assert!(battle_save_path().ends_with("battle.ron"));
        assert!(campaign_save_path().ends_with("campaign.ron"));
    }

    #[test]
    fn full_two_ship_fidelity_roundtrip() {
        let design = ShipDesign::new(vec![
            BlockPlacement::new(IVec2::ZERO, "core", Rot90::R0),
            BlockPlacement::new(IVec2::X, "gun", Rot90::R0),
        ]);

        let mut source = App::new();
        source
            .insert_resource(registry())
            .insert_resource(GameConfigValue(GameConfig::default()))
            .init_resource::<CameraTarget>()
            .init_resource::<ControlledShip>()
            .init_resource::<ControlMode>()
            .add_systems(Startup, {
                let design = design.clone();
                move |mut commands: Commands,
                      reg: Res<BlockRegistry>,
                      cfg: Res<GameConfigValue>,
                      mut cam_target: ResMut<CameraTarget>,
                      mut controlled: ResMut<ControlledShip>,
                      mut mode: ResMut<ControlMode>| {
                    let (ship_a, _) = design
                        .spawn(&mut commands, &reg, Team::new(0), Transform::from_xyz(10.0, 20.0, 0.0), &cfg)
                        .unwrap();
                    let (ship_b, _) = design
                        .spawn(&mut commands, &reg, Team::new(1), Transform::from_xyz(-30.0, 40.0, 0.0), &cfg)
                        .unwrap();
                    cam_target.following = Some(ship_a);
                    controlled.controlling = Some(ship_a);
                    *mode = ControlMode::Pilot;
                    commands.entity(ship_b).insert(AiRuntime {
                        current: AiBehavior::Attack { target: Some(ship_a) },
                        spawn_position: Vec2::new(-30.0, 40.0),
                        rule_cooldowns: vec![3.5],
                        think_timer: Timer::from_seconds(0.2, TimerMode::Repeating),
                    });
                    commands.spawn((
                        Loot::Item(ItemDrop {
                            item_type: "steel".to_string(),
                            amount: 15,
                        }),
                        Transform::from_xyz(5.0, 6.0, 0.0),
                        Lifetime::seconds(12.0),
                    ));

                    let debris = commands
                        .spawn((
                            Debris,
                            Team::new(1),
                            Transform::from_xyz(7.0, 8.0, 0.0),
                            Velocity {
                                linear: Vec2::new(1.0, 2.0),
                                angular: 0.5,
                            },
                            Lifetime::seconds(9.0),
                        ))
                        .id();
                    let placement = BlockPlacement::new(IVec2::ZERO, "gun", Rot90::R0);
                    let def = reg.get_symbol("gun").expect("gun is registered");
                    let mut debris_grid = BlockGrid::new();
                    debris_grid.insert_block(placement.clone(), Entity::PLACEHOLDER, def);
                    let block = spawn_block_entity(
                        &mut commands,
                        &placement,
                        def,
                        debris,
                        Team::new(1),
                        &cfg,
                    );
                    debris_grid.set_entity(placement.id, block);
                    commands.entity(debris).insert(debris_grid);
                }
            });
        source.update();

        // Mutate player ship cargo, battery charge, turret charge, barrel angle, and weapon cooldown
        let player_entity = source.world().resource::<ControlledShip>().controlling.unwrap();
        {
            let mut stats = source.world_mut().get_mut::<ShipStats>(player_entity).unwrap();
            stats.cargo_used = 25;
            stats.cargo_items.insert("steel".to_string(), 25);
        }
        let block_entities: Vec<Entity> = source
            .world()
            .get::<BlockGrid>(player_entity)
            .unwrap()
            .blocks()
            .map(|b| b.entity)
            .collect();
        for block_entity in block_entities {
            if let Some(mut charge) = source.world_mut().get_mut::<Charge>(block_entity) {
                charge.value = 42.0;
            }
            if let Some(mut t_charge) = source.world_mut().get_mut::<TurretCharge>(block_entity) {
                t_charge.value = 7.5;
            }
            if let Some(mut single) =
                source.world_mut().get_mut::<SingleShootPatternState>(block_entity)
            {
                single.cooldown_remaining = 0.65;
            }
            let barrel = source.world().get::<TurretBarrel>(block_entity).map(|b| b.0);
            if let Some(barrel_entity) = barrel
                && let Some(mut tf) = source.world_mut().get_mut::<Transform>(barrel_entity)
            {
                tf.rotation = Quat::from_rotation_z(0.75);
            }
        }

        let ctx = CaptureContext {
            world: source.world(),
            mission: "fidelity-test",
            elapsed_secs: 19.0,
            outcome: None,
            saved_at_unix_ms: 1_700_000_000_000,
            seed: 999,
        };
        let save = WorldSave::capture(&ctx);
        let ship_ids: Vec<Uuid> = save
            .bodies
            .iter()
            .filter_map(|body| match body {
                BodySnapshot::Ship(ship) => Some(ship.id),
                BodySnapshot::Debris(_) => None,
            })
            .collect();
        assert_eq!(ship_ids.len(), 2);
        assert_eq!(save.bodies.len(), 3, "two ships and one debris body");
        assert_ne!(ship_ids[0], ship_ids[1], "identical designs must produce distinct ship instance IDs");

        let mut restored = App::new();
        let restore_save = save.clone();
        restored
            .insert_resource(registry())
            .insert_resource(GameConfigValue(GameConfig::default()))
            .add_systems(
                Startup,
                move |mut commands: Commands, reg: Res<BlockRegistry>, cfg: Res<GameConfigValue>| {
                    restore_save
                        .clone()
                        .restore(&mut commands, &reg, &cfg, None)
                        .expect("snapshot restores");
                },
            );
        restored.update();

        let world = restored.world_mut();
        let controlled_entity = world.resource::<ControlledShip>().controlling.expect("controlled ship restored");
        assert_eq!(world.resource::<CameraTarget>().following, Some(controlled_entity));
        assert_eq!(*world.resource::<ControlMode>(), ControlMode::Pilot);

        let stats = world.get::<ShipStats>(controlled_entity).unwrap();
        assert_eq!(stats.cargo_used, 25);
        assert_eq!(stats.cargo_items.get("steel"), Some(&25));

        let mut enemy_ai_checked = false;
        let mut ship_query = world.query::<(Entity, &Team, &AiRuntime, &BlockGrid)>();
        for (entity, team, ai_runtime, ship_grid) in ship_query.iter(world) {
            if !team.is_player() {
                assert_eq!(ai_runtime.current, AiBehavior::Attack { target: Some(controlled_entity) });
                assert_eq!(ai_runtime.spawn_position, Vec2::new(-30.0, 40.0));
                assert_eq!(ai_runtime.rule_cooldowns, vec![3.5]);
                enemy_ai_checked = true;
            } else {
                assert_eq!(entity, controlled_entity);
                for b in ship_grid.blocks() {
                    if let Some(charge) = world.get::<Charge>(b.entity) {
                        assert_eq!(charge.value, 42.0);
                    }
                    if let Some(t_charge) = world.get::<TurretCharge>(b.entity) {
                        assert_eq!(t_charge.value, 7.5);
                    }
                    if let Some(single) = world.get::<SingleShootPatternState>(b.entity) {
                        assert!((single.cooldown_remaining - 0.65).abs() < 1e-5);
                    }
                    if let Some(barrel) = world.get::<TurretBarrel>(b.entity) {
                        let rot = world.get::<Transform>(barrel.0).unwrap().rotation.to_euler(EulerRot::ZYX).0;
                        assert!((rot - 0.75).abs() < 1e-4);
                    }
                }
            }
        }
        assert!(enemy_ai_checked);

        let mut drops = world.query::<(&Loot, &Sprite)>();
        assert_eq!(drops.iter(world).count(), 1, "restored drop must include a visible Sprite");

        let mut debris_query = world.query::<(&Debris, &Team, &BlockGrid, &Lifetime)>();
        let (_, team, grid, lifetime) = debris_query
            .iter(world)
            .next()
            .expect("debris body restored");
        assert_eq!(team.id, 1);
        assert_eq!(grid.blocks.len(), 1);
        assert!(grid.blocks().all(|block| block.entity != Entity::PLACEHOLDER));
        assert!((lifetime.timer().remaining_secs() - 9.0).abs() < 1e-4);
    }
}
