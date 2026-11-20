use bevy::prelude::*;

use super::*;
use crate::config::GameConfigValue;
use crate::constant::{FLASH_OVERLAY_COLOR, LAYER_DROP, LAYER_FLASH_OVERLAY};
use crate::content::damage::DamageEvent;
use crate::content::item::ItemRegistry;
use crate::content::lifetime::Lifetime;
use crate::content::body::{BlockGrid, BodyMutation, Debris};
use crate::resource::SceneState;


pub fn handle_block_destroy(
    mut writer: MessageWriter<BodyMutation>,
    query: Query<(&Health, &BlockRef, &BlockOwner)>,
) {
    for (health, block, owner) in query.iter() {
        if health.health <= 0.0 {
            writer.write(BodyMutation::DestroyBlock {
                body: owner.body,
                block_id: block.id,
            });
        }
    }
}

pub fn handle_block_damage_event(
    mut events: MessageReader<DamageEvent>,
    mut commands: Commands,
    blocks: Query<(&BlockRef, &BlockOwner)>,
    grids: Query<&BlockGrid>,
    registry: Res<BlockRegistry>,
) {
    for event in events.read() {
        let Ok((block, owner)) = blocks.get(event.target) else {
            continue;
        };
        let Ok(grid) = grids.get(owner.body) else {
            continue;
        };
        let Some(grid_block) = grid.block(block.id) else {
            continue;
        };
        let Some(def) = registry.get_symbol(&grid_block.placement.block_type) else {
            continue;
        };

        let overlay = commands
            .spawn((
                Sprite {
                    color: FLASH_OVERLAY_COLOR,
                    custom_size: Some(def.get_base_pixel_size()),
                    ..default()
                },
                FlashOverlay {
                    timer: Timer::from_seconds(0.1, TimerMode::Once),
                },
                Transform::from_translation(Vec2::ZERO.extend(LAYER_FLASH_OVERLAY)),
            ))
            .id();

        commands.entity(event.target).add_child(overlay);
    }
}

pub fn handle_flash_overlay(
    mut commands: Commands,
    mut flash_overlays: Query<(&mut FlashOverlay, Entity)>,
    time: Res<Time>,
) {
    for (mut flash_overlay, entity) in flash_overlays.iter_mut() {
        flash_overlay.timer.tick(time.delta());
        if flash_overlay.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn handle_drop(
    mut commands: Commands,
    mut events: MessageReader<DropEvent>,
    item_registry: Res<ItemRegistry>,
    config: Res<GameConfigValue>,
) {
    for event in events.read() {
        let rarity = match &event.drop {
            Loot::Item(item) => item_registry.get(&item.item_type).map(|f| f.rarity),
        };

        commands.spawn((
            event.drop.clone(),
            DespawnOnExit(SceneState::Game),
            Transform::from_translation(event.position.extend(LAYER_DROP)),
            Sprite {
                color: rarity
                    .map(|f| f.color())
                    .unwrap_or(Color::srgb(5.0, 5.0, 5.0)),
                custom_size: Some(Vec2::new(config.0.drop_size, config.0.drop_size)),
                ..default()
            },
            Lifetime::seconds(config.0.drop_despawn_in_seconds),
        ));
    }
}

pub fn sync_debris_disabled(
    trigger: On<Insert<BlockOwner>>,
    owners: Query<&BlockOwner>,
    debris: Query<(), With<Debris>>,
    mut commands: Commands,
) {
    let block = trigger.event().entity;
    if let Ok(owner) = owners.get(block)
        && debris.contains(owner.body)
    {
        commands.entity(block).insert(Disabled);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GameConfigValue;
    use crate::content::block::component::{
        LaserBeamPatternShootingState, LaserBeamPatternState, PullingBeamPatternState,
        ShootPattern, ShootingProjectile, SingleShootPattern, SingleShootPatternState,
        TargetingStrategy,
    };
    use crate::content::block::feature::power::{
        Booster, BoosterState, Demand,
        booster::{power_boosters, reset_disabled_booster},
        grid::{GridScratch, ShipPowerScratch, reset_disabled_demand},
    };
    use crate::content::block::feature::turret::{
        Turret, TurretBarrel, TurretCharge, TurretPower, TurretSprite, TurretState, TurretTarget,
        advance_turret_charge, reset_disabled_turret, turret_tracking, update_turret_demand,
        update_turret_target,
    };
    use crate::content::block::radar::{Radar, reset_disabled_radar};
    use crate::content::bullet::{Bullet, handle_single_shot, reset_disabled_beams};
    use crate::content::ship::{Ship, ShipStats};
    use crate::content::team::Team;

    #[test]
    fn debris_owner_automatically_disables_block_and_resets_state() {
        let mut app = App::new();
        app.add_observer(sync_debris_disabled)
            .add_observer(reset_disabled_radar)
            .add_observer(reset_disabled_turret)
            .add_observer(reset_disabled_booster)
            .add_observer(reset_disabled_demand)
            .add_observer(reset_disabled_beams);

        let ship = app.world_mut().spawn(Ship::default()).id();
        let debris = app.world_mut().spawn(Debris).id();
        let laser_beam = app.world_mut().spawn(Visibility::Visible).id();
        let pulling_beam = app.world_mut().spawn(Visibility::Visible).id();

        let mut radar = Radar::new(100.0, 1.0);
        radar.targets.push(ship);

        let block = app
            .world_mut()
            .spawn((
                BlockOwner { body: ship },
                TurretState::Ready { aim_point: Vec2::new(10.0, 20.0), target: None },
                radar,
                BoosterState { powered: true },
                Demand { value: 5.0 },
                LaserBeamPatternState {
                    beam: Some(laser_beam),
                    shooting: LaserBeamPatternShootingState::Shooting { elapsed: 0.5 },
                },
                PullingBeamPatternState {
                    beam: Some(pulling_beam),
                },
            ))
            .id();

        app.update();
        assert!(app.world().get::<Disabled>(block).is_none());

        app.world_mut()
            .entity_mut(block)
            .insert(BlockOwner { body: debris });
        app.update();

        assert!(app.world().get::<Disabled>(block).is_some());
        let turret_state = app.world().get::<TurretState>(block).unwrap();
        assert!(!turret_state.active());
        assert!(!turret_state.active());
        assert!(turret_state.target().is_none());
        assert!(app.world().get::<Radar>(block).unwrap().targets.is_empty());
        assert!(!app.world().get::<BoosterState>(block).unwrap().powered);
        assert_eq!(app.world().get::<Demand>(block).unwrap().value, 0.0);
        assert!(matches!(
            app.world().get::<LaserBeamPatternState>(block).unwrap().shooting,
            LaserBeamPatternShootingState::Cooldown { .. }
        ));
        assert_eq!(
            *app.world().get::<Visibility>(laser_beam).unwrap(),
            Visibility::Hidden
        );
        assert_eq!(
            *app.world().get::<Visibility>(pulling_beam).unwrap(),
            Visibility::Hidden
        );
    }

    #[test]
    fn disabled_turret_does_not_target_track_charge_or_fire() {
        use std::time::Duration;

        let mut app = App::new();
        app.insert_resource(Time::<()>::default());
        app.insert_resource(GameConfigValue::default());
        app.add_systems(
            Update,
            (
                update_turret_demand,
                advance_turret_charge,
                update_turret_target,
                turret_tracking,
                handle_single_shot,
            )
                .chain(),
        );

        let enemy_ship = app
            .world_mut()
            .spawn((
                Ship::default(),
                BlockGrid::new(),
                ShipStats {
                    health: 100.0,
                    ..default()
                },
                GlobalTransform::from_translation(Vec3::new(50.0, 0.0, 0.0)),
            ))
            .id();

        let barrel = app
            .world_mut()
            .spawn((
                TurretSprite,
                Transform::default(),
                GlobalTransform::default(),
            ))
            .id();

        let mut radar = Radar::new(500.0, 1.0);
        radar.targets.push(enemy_ship);

        let turret = app
            .world_mut()
            .spawn((
                Disabled,
                Turret {
                    range: 500.0,
                    turn_speed: 5.0,
                    aim_tolerance: 0.5,
                    barrels: vec![Vec2::ZERO],
                    strategy: TargetingStrategy::Closest,
                    shooting_pattern: ShootPattern::Single(SingleShootPattern {
                        fire_rate: 0.5,
                        projectile: ShootingProjectile::Bullet {
                            speed: 100.0,
                            damage: 10.0,
                            lifetime: 2.0,
                            piercing: 0,
                            width: 4.0,
                            length: 8.0,
                        },
                    }),
                },
                TurretState::Ready {
                    aim_point: Vec2::new(50.0, 0.0),
                    target: Some(TurretTarget::Ship(enemy_ship)),
                },
                SingleShootPattern {
                    fire_rate: 0.5,
                    projectile: ShootingProjectile::Bullet {
                        speed: 100.0,
                        damage: 10.0,
                        lifetime: 2.0,
                        piercing: 0,
                        width: 4.0,
                        length: 8.0,
                    },
                },
                SingleShootPatternState {
                    cooldown_remaining: 0.0,
                },
                TurretPower::Charge {
                    capacity: 10.0,
                    charge_rate: 5.0,
                },
                TurretCharge { value: 2.0 },
                Demand::default(),
                Team::new(0),
                radar,
                TurretBarrel(barrel),
            ))
            .id();

        app.world_mut()
            .resource_mut::<Time<()>>()
            .advance_by(Duration::from_secs(1));
        app.update();

        assert_eq!(app.world().get::<Demand>(turret).unwrap().value, 0.0);
        assert_eq!(app.world().get::<TurretCharge>(turret).unwrap().value, 2.0);
        assert_eq!(
            app.world().get::<Transform>(barrel).unwrap().rotation,
            Quat::IDENTITY
        );
        let mut bullets = app.world_mut().query::<&Bullet>();
        assert_eq!(bullets.iter(app.world()).count(), 0);
    }

    #[test]
    fn disabled_booster_is_not_powered() {
        let mut app = App::new();
        app.insert_resource(Time::<()>::default());
        app.insert_resource(GridScratch::default());
        app.add_systems(Update, power_boosters);

        let ship = app.world_mut().spawn(Ship::default()).id();
        app.world_mut()
            .resource_mut::<GridScratch>()
            .ships
            .insert(
                ship,
                ShipPowerScratch {
                    generation: 100.0,
                    ..default()
                },
            );

        let booster = app
            .world_mut()
            .spawn((
                Disabled,
                BlockOwner { body: ship },
                Transform::default(),
                Booster {
                    radius: 100.0,
                    boost: 2.0,
                    rate: 5.0,
                },
                BoosterState { powered: false },
            ))
            .id();

        app.update();
        assert!(!app.world().get::<BoosterState>(booster).unwrap().powered);
    }
}
