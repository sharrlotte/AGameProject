use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use data::{
    ConfigHotReloadedMessage, DamageMessage, EnemyTag, GROUP_ENEMY, GROUP_PROJECTILE, GROUP_WORLD,
    GameState, ObstacleTag, PlayerConfig, RunEntity, SimPostPhysicsSet, SimUpdateSet,
    UpgradeModifiers, WeaponFiredMessage, WeaponSpecData, WeaponsConfig,
};
use sim_world::{ActionIntent, AimDirection, Stamina, Survivor};

#[derive(Debug, Clone)]
pub struct WeaponSlotState {
    pub spec: WeaponSpecData,
    pub ammo: u32,
}

impl WeaponSlotState {
    fn new(spec: WeaponSpecData) -> Self {
        let ammo = spec.mag_size;
        Self { spec, ammo }
    }
}

/// Holds the survivor's equipped weapons and reload/fire cooldown state.
#[derive(Component, Debug, Clone)]
pub struct WeaponInventory {
    pub slots: Vec<WeaponSlotState>,
    pub active_slot: usize,
    pub cooldown_timer: f32,
    pub reload_timer: Option<f32>,
    pub reload_total: f32,
}

impl WeaponInventory {
    pub fn from_config(cfg: &WeaponsConfig) -> Self {
        let slots = cfg
            .weapons
            .iter()
            .cloned()
            .map(WeaponSlotState::new)
            .collect();
        Self {
            slots,
            active_slot: 0,
            cooldown_timer: 0.0,
            reload_timer: None,
            reload_total: 0.0,
        }
    }

    pub fn active(&self) -> Option<&WeaponSlotState> {
        self.slots.get(self.active_slot)
    }

    pub fn active_mut(&mut self) -> Option<&mut WeaponSlotState> {
        self.slots.get_mut(self.active_slot)
    }
}

/// Physical 3D projectile sensor entity fired by weapons.
#[derive(Component, Debug, Clone, Copy)]
pub struct Projectile {
    pub owner: Entity,
    pub damage: f32,
    pub knockback: f32,
    pub direction: Vec3,
    pub radius: f32,
    pub lifetime: f32,
    pub hit: bool,
}

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                attach_weapon_inventory,
                sync_weapons_hot_reload,
                handle_weapons,
                tick_projectiles,
            )
                .chain()
                .in_set(SimUpdateSet)
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            PostUpdate,
            handle_projectile_collisions
                .in_set(SimPostPhysicsSet)
                .after(PhysicsSet::Writeback)
                .before(TransformSystems::Propagate)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

fn attach_weapon_inventory(
    mut commands: Commands,
    cfg: Res<WeaponsConfig>,
    new_survivors: Query<Entity, (With<Survivor>, Without<WeaponInventory>)>,
) {
    for entity in &new_survivors {
        commands
            .entity(entity)
            .insert(WeaponInventory::from_config(&cfg));
    }
}

fn sync_weapons_hot_reload(
    mut reloaded: MessageReader<ConfigHotReloadedMessage>,
    cfg: Res<WeaponsConfig>,
    mut inventories: Query<&mut WeaponInventory>,
) {
    if reloaded.read().next().is_none() || !cfg.is_changed() || cfg.weapons.is_empty() {
        return;
    }

    for mut inv in &mut inventories {
        for (idx, spec) in cfg.weapons.iter().enumerate() {
            if let Some(slot) = inv.slots.get_mut(idx) {
                slot.spec = spec.clone();
                slot.ammo = slot.ammo.min(spec.mag_size);
            } else {
                inv.slots.push(WeaponSlotState::new(spec.clone()));
            }
        }
        inv.slots.truncate(cfg.weapons.len());
        if inv.active_slot >= inv.slots.len() {
            inv.active_slot = 0;
        }
    }
}

/// Processes weapon swapping, reloading, and firing from `ActionIntent`.
pub fn handle_weapons(
    mut commands: Commands,
    time: Res<Time>,
    weapons_cfg: Res<WeaponsConfig>,
    player_cfg: Res<PlayerConfig>,
    upgrades: Res<UpgradeModifiers>,
    mut fired_writer: MessageWriter<WeaponFiredMessage>,
    mut shooters: Query<
        (
            Entity,
            &Transform,
            &AimDirection,
            &Stamina,
            &mut ActionIntent,
            &mut WeaponInventory,
        ),
        With<Survivor>,
    >,
) {
    let dt = time.delta_secs();
    let reload_speed = upgrades.reload_speed_multiplier.max(0.1);

    for (entity, transform, aim_dir, stamina, mut intent, mut inventory) in &mut shooters {
        inventory.cooldown_timer = (inventory.cooldown_timer - dt).max(0.0);

        if let Some(target_slot) = intent.swap_slot.take() {
            if target_slot < inventory.slots.len() && target_slot != inventory.active_slot {
                inventory.active_slot = target_slot;
                inventory.reload_timer = None;
                inventory.reload_total = 0.0;
                inventory.cooldown_timer = inventory
                    .cooldown_timer
                    .min(weapons_cfg.swap_cooldown_cap);
            }
        }

        if let Some(remaining) = inventory.reload_timer.as_mut() {
            *remaining -= dt;
            if *remaining <= 0.0 {
                if let Some(active) = inventory.active_mut() {
                    active.ammo = active.spec.mag_size;
                }
                inventory.reload_timer = None;
                inventory.reload_total = 0.0;
            }
        }

        let want_reload = intent.reload_just_pressed;
        intent.reload_just_pressed = false;
        if want_reload && inventory.reload_timer.is_none() {
            if let Some(active) = inventory.active() {
                if active.ammo < active.spec.mag_size {
                    let duration = active.spec.reload_secs / reload_speed;
                    inventory.reload_total = duration;
                    inventory.reload_timer = Some(duration);
                }
            }
        }

        let fire_just_pressed = intent.fire_just_pressed;
        intent.fire_just_pressed = false;

        if inventory.reload_timer.is_some() || inventory.cooldown_timer > 0.0 {
            continue;
        }

        let Some(active_spec) = inventory.active().map(|s| s.spec.clone()) else {
            continue;
        };
        let active_ammo = inventory.active().map(|s| s.ammo).unwrap_or(0);

        let triggered = if active_spec.full_auto {
            intent.fire_held
        } else {
            fire_just_pressed
        };

        if !triggered {
            continue;
        }

        if active_ammo == 0 {
            let duration = active_spec.reload_secs / reload_speed;
            inventory.reload_total = duration;
            inventory.reload_timer = Some(duration);
            continue;
        }

        if let Some(active) = inventory.active_mut() {
            active.ammo -= 1;
        }
        inventory.cooldown_timer = active_spec.fire_interval;

        let forward = if aim_dir.0.length_squared() > 1e-4 {
            aim_dir.0.normalize()
        } else {
            Vec3::NEG_Z
        };
        let muzzle_pos = Vec3::new(
            transform.translation.x,
            player_cfg.aim_height,
            transform.translation.z,
        ) + forward * weapons_cfg.muzzle_offset;

        let spread_mult = if stamina.is_sprinting {
            weapons_cfg.sprint_spread_multiplier
        } else {
            1.0
        };
        let half_spread_rad = (active_spec.spread_deg * spread_mult).to_radians() * 0.5;
        let scaled_damage = active_spec.damage * upgrades.damage_multiplier;

        for _ in 0..active_spec.pellets {
            let angle = if half_spread_rad > 0.0 {
                rand::random_range(-half_spread_rad..=half_spread_rad)
            } else {
                0.0
            };
            let dir = (Quat::from_rotation_y(angle) * forward).normalize();

            commands.spawn((
                Name::new("Projectile"),
                RunEntity,
                Projectile {
                    owner: entity,
                    damage: scaled_damage,
                    knockback: active_spec.knockback,
                    direction: dir,
                    radius: active_spec.projectile_radius,
                    lifetime: active_spec.projectile_lifetime,
                    hit: false,
                },
                RigidBody::Dynamic,
                GravityScale(0.0),
                Ccd::enabled(),
                Sensor,
                ActiveEvents::COLLISION_EVENTS,
                Collider::ball(active_spec.projectile_radius),
                Velocity::linear(dir * active_spec.projectile_speed),
                TransformInterpolation::default(),
                LockedAxes::TRANSLATION_LOCKED_Y | LockedAxes::ROTATION_LOCKED,
                CollisionGroups::new(GROUP_PROJECTILE, GROUP_WORLD | GROUP_ENEMY),
                Transform::from_translation(muzzle_pos),
            ));
        }

        fired_writer.write(WeaponFiredMessage {
            shooter: entity,
            origin: muzzle_pos,
            direction: forward,
        });
    }
}

/// Resolves sensor collision events for 3D projectiles against enemies and arena obstacles.
pub fn handle_projectile_collisions(
    mut commands: Commands,
    mut collision_reader: MessageReader<CollisionEvent>,
    mut damage_writer: MessageWriter<DamageMessage>,
    mut projectiles: Query<&mut Projectile>,
    enemies: Query<(), With<EnemyTag>>,
    obstacles: Query<(), With<ObstacleTag>>,
) {
    for event in collision_reader.read() {
        let CollisionEvent::Started(e1, e2, _) = *event else {
            continue;
        };

        let (proj_entity, other_entity) = if projectiles.contains(e1) {
            (e1, e2)
        } else if projectiles.contains(e2) {
            (e2, e1)
        } else {
            continue;
        };

        let Ok(mut proj) = projectiles.get_mut(proj_entity) else {
            continue;
        };
        if proj.hit {
            continue;
        }

        if enemies.contains(other_entity) {
            proj.hit = true;
            damage_writer.write(DamageMessage {
                target: other_entity,
                amount: proj.damage,
                knockback: proj.direction * proj.knockback,
                source: proj.owner,
            });
            commands.entity(proj_entity).despawn();
        } else if obstacles.contains(other_entity) {
            proj.hit = true;
            commands.entity(proj_entity).despawn();
        }
    }
}

/// Despawns projectiles whose maximum flight lifetime has elapsed.
pub fn tick_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    mut projectiles: Query<(Entity, &mut Projectile)>,
) {
    let dt = time.delta_secs();
    for (entity, mut proj) in &mut projectiles {
        proj.lifetime -= dt;
        if proj.lifetime <= 0.0 && !proj.hit {
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use sim_world::player::spawn_player;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<WeaponsConfig>()
            .init_resource::<PlayerConfig>()
            .init_resource::<UpgradeModifiers>()
            .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
                Duration::from_secs_f32(1.0 / 60.0),
            ))
            .add_message::<WeaponFiredMessage>()
            .add_message::<ConfigHotReloadedMessage>()
            .add_systems(Startup, spawn_player)
            .add_systems(
                Update,
                (attach_weapon_inventory, handle_weapons).chain(),
            );
        app
    }

    fn step_sim(app: &mut App, secs: f32) {
        let steps = (secs / (1.0 / 60.0)).ceil().max(1.0) as usize;
        for _ in 0..steps {
            app.update();
        }
    }

    #[test]
    fn rifle_fires_shotgun_swaps_and_reload_cancels_on_swap() {
        let mut app = test_app();
        app.update();

        let player = app
            .world_mut()
            .query_filtered::<Entity, With<Survivor>>()
            .single(app.world())
            .expect("player spawned");

        let cfg = app.world().resource::<WeaponsConfig>().clone();
        let rifle_mag = cfg.weapons[0].mag_size;
        let shotgun_pellets = cfg.weapons[1].pellets;

        app.world_mut()
            .get_mut::<ActionIntent>(player)
            .unwrap()
            .fire_held = true;
        step_sim(&mut app, 0.05);

        let proj_count = app
            .world_mut()
            .query::<&Projectile>()
            .iter(app.world())
            .count();
        assert_eq!(proj_count, 1);
        assert_eq!(
            app.world()
                .get::<WeaponInventory>(player)
                .unwrap()
                .active()
                .unwrap()
                .ammo,
            rifle_mag - 1
        );

        {
            let mut intent = app.world_mut().get_mut::<ActionIntent>(player).unwrap();
            intent.fire_held = false;
            intent.reload_just_pressed = true;
        }
        step_sim(&mut app, 0.05);
        assert!(
            app.world()
                .get::<WeaponInventory>(player)
                .unwrap()
                .reload_timer
                .is_some()
        );

        {
            let mut intent = app.world_mut().get_mut::<ActionIntent>(player).unwrap();
            intent.swap_slot = Some(1);
        }
        step_sim(&mut app, 0.20);
        {
            let inv = app.world().get::<WeaponInventory>(player).unwrap();
            assert_eq!(inv.active_slot, 1);
            assert!(inv.reload_timer.is_none());
        }

        app.world_mut()
            .get_mut::<ActionIntent>(player)
            .unwrap()
            .fire_just_pressed = true;
        step_sim(&mut app, 0.05);

        let total_projs = app
            .world_mut()
            .query::<&Projectile>()
            .iter(app.world())
            .count();
        assert_eq!(total_projs, 1 + shotgun_pellets as usize);
    }

    #[test]
    fn empty_trigger_starts_auto_reload_and_refills_mag() {
        let mut app = test_app();
        app.update();

        let player = app
            .world_mut()
            .query_filtered::<Entity, With<Survivor>>()
            .single(app.world())
            .expect("player spawned");

        let cfg = app.world().resource::<WeaponsConfig>().clone();
        let rifle = &cfg.weapons[0];

        app.world_mut()
            .get_mut::<WeaponInventory>(player)
            .unwrap()
            .active_mut()
            .unwrap()
            .ammo = 0;

        app.world_mut()
            .get_mut::<ActionIntent>(player)
            .unwrap()
            .fire_held = true;
        step_sim(&mut app, 0.05);

        assert!(
            app.world()
                .get::<WeaponInventory>(player)
                .unwrap()
                .reload_timer
                .is_some()
        );

        app.world_mut()
            .get_mut::<ActionIntent>(player)
            .unwrap()
            .fire_held = false;
        step_sim(&mut app, rifle.reload_secs + 0.1);

        let inv = app.world().get::<WeaponInventory>(player).unwrap();
        assert!(inv.reload_timer.is_none());
        assert_eq!(inv.active().unwrap().ammo, rifle.mag_size);
    }
}
