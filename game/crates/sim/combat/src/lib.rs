use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use data::{
    AppRonConfigExt, ConfigHotReloadedMessage, DamageMessage, EnemyTag, GROUP_ENEMY,
    GROUP_PROJECTILE, GROUP_WORLD, GameState, ObstacleTag, PlayerDiedMessage, ReloadCancelReason,
    ReloadCancelledMessage, ReloadFinishedMessage, ReloadStartedMessage, RunEntity,
    SimPostPhysicsSet, SimUpdateSet, UpgradeModifiers, WeaponFiredMessage, WeaponSwappedMessage,
};
use world::{ActionIntent, AimDirection, Health, PlayerConfig, Stamina, Survivor};

pub mod config;
pub use config::{WeaponSpecData, WeaponsConfig};

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

    /// Normalized reload progress in `0..=1`, or `None` when no reload is active.
    pub fn reload_progress(&self) -> Option<f32> {
        let remaining = self.reload_timer?;
        if self.reload_total <= 0.0 {
            return Some(0.0);
        }
        Some((1.0 - remaining / self.reload_total).clamp(0.0, 1.0))
    }
}

/// Simulation-owned combat stance consumed by animation and future gameplay modifiers.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct CombatStance {
    pub aiming: bool,
    /// Remaining seconds the stance stays raised after the last triggering action ends.
    pub hold_timer: f32,
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
        app.register_ron_config::<WeaponsConfig>()
            .add_systems(
                Update,
                (
                    attach_weapon_inventory,
                    sync_weapons_hot_reload,
                    handle_weapons,
                    update_combat_stance,
                    tick_projectiles,
                )
                    .chain()
                    .in_set(SimUpdateSet)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                PostUpdate,
                (
                    handle_projectile_collisions
                        .after(PhysicsSet::Writeback)
                        .before(TransformSystems::Propagate),
                    cancel_reload_on_death.after(world::player::apply_player_damage),
                )
                    .in_set(SimPostPhysicsSet)
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
            .insert((WeaponInventory::from_config(&cfg), CombatStance::default()));
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
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn handle_weapons(
    mut commands: Commands,
    time: Res<Time>,
    weapons_cfg: Res<WeaponsConfig>,
    player_cfg: Res<PlayerConfig>,
    upgrades: Res<UpgradeModifiers>,
    mut fired_writer: MessageWriter<WeaponFiredMessage>,
    mut started_writer: MessageWriter<ReloadStartedMessage>,
    mut finished_writer: MessageWriter<ReloadFinishedMessage>,
    mut cancelled_writer: MessageWriter<ReloadCancelledMessage>,
    mut swapped_writer: MessageWriter<WeaponSwappedMessage>,
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

        if let Some(target_slot) = intent.swap_slot.take()
            && target_slot < inventory.slots.len()
            && target_slot != inventory.active_slot
        {
            let from = inventory.active_slot;
            let was_reloading = inventory.reload_timer.is_some();
            inventory.active_slot = target_slot;
            inventory.reload_timer = None;
            inventory.reload_total = 0.0;
            inventory.cooldown_timer = inventory.cooldown_timer.min(weapons_cfg.swap_cooldown_cap);
            swapped_writer.write(WeaponSwappedMessage {
                entity,
                from,
                to: target_slot,
            });
            if was_reloading {
                cancelled_writer.write(ReloadCancelledMessage {
                    entity,
                    reason: ReloadCancelReason::Swapped,
                });
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
                finished_writer.write(ReloadFinishedMessage { entity });
            }
        }

        let want_reload = intent.reload_just_pressed;
        intent.reload_just_pressed = false;
        if want_reload
            && inventory.reload_timer.is_none()
            && let Some(active) = inventory.active()
            && active.ammo < active.spec.mag_size
        {
            let duration = active.spec.reload_secs / reload_speed;
            inventory.reload_total = duration;
            inventory.reload_timer = Some(duration);
            started_writer.write(ReloadStartedMessage { entity, duration });
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
            started_writer.write(ReloadStartedMessage { entity, duration });
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

/// Derives the combat stance from aim, fire, and reload state.
///
/// The hold timer keeps the arms raised between shots and after the reload ends,
/// so the upper-body layer never flickers. Dead players are never in stance.
pub fn update_combat_stance(
    time: Res<Time>,
    cfg: Res<WeaponsConfig>,
    mut stances: Query<
        (&ActionIntent, &WeaponInventory, &Health, &mut CombatStance),
        With<Survivor>,
    >,
) {
    let dt = time.delta_secs();
    let hold = cfg.stance_hold_secs.max(0.0);

    for (intent, inventory, health, mut stance) in &mut stances {
        if health.current <= 0.0 {
            stance.aiming = false;
            stance.hold_timer = 0.0;
            continue;
        }

        // Reload owns the arms for its whole duration, independent of movement.
        if intent.aim_held || intent.fire_held || inventory.reload_timer.is_some() {
            stance.hold_timer = hold;
            stance.aiming = true;
        } else if stance.hold_timer > 0.0 {
            stance.hold_timer = (stance.hold_timer - dt).max(0.0);
            stance.aiming = stance.hold_timer > 0.0;
        } else {
            stance.aiming = false;
        }
    }
}

/// Cancels an in-flight reload when the survivor dies and drops the stance.
fn cancel_reload_on_death(
    mut died_reader: MessageReader<PlayerDiedMessage>,
    mut cancelled_writer: MessageWriter<ReloadCancelledMessage>,
    mut inventories: Query<&mut WeaponInventory>,
    mut stances: Query<&mut CombatStance>,
) {
    for msg in died_reader.read() {
        if let Ok(mut inventory) = inventories.get_mut(msg.survivor)
            && inventory.reload_timer.is_some()
        {
            inventory.reload_timer = None;
            inventory.reload_total = 0.0;
            cancelled_writer.write(ReloadCancelledMessage {
                entity: msg.survivor,
                reason: ReloadCancelReason::Died,
            });
        }
        if let Ok(mut stance) = stances.get_mut(msg.survivor) {
            stance.aiming = false;
            stance.hold_timer = 0.0;
        }
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
    use world::player::spawn_player;

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
            .add_message::<ReloadStartedMessage>()
            .add_message::<ReloadFinishedMessage>()
            .add_message::<ReloadCancelledMessage>()
            .add_message::<WeaponSwappedMessage>()
            .add_message::<ConfigHotReloadedMessage>()
            .add_message::<PlayerDiedMessage>()
            .add_systems(Startup, spawn_player)
            .add_systems(
                Update,
                (
                    attach_weapon_inventory,
                    handle_weapons,
                    update_combat_stance,
                    cancel_reload_on_death,
                )
                    .chain(),
            );
        app
    }

    fn messages<T: Message + Clone>(app: &App) -> Vec<T> {
        let msgs = app.world().resource::<Messages<T>>();
        let mut cursor = msgs.get_cursor();
        cursor.read(msgs).cloned().collect()
    }

    fn player_entity(app: &mut App) -> Entity {
        app.world_mut()
            .query_filtered::<Entity, With<Survivor>>()
            .single(app.world())
            .expect("player spawned")
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

    #[test]
    fn stance_enters_from_aim_fire_and_reload_and_exits_after_hold() {
        let mut app = test_app();
        app.update();
        let player = player_entity(&mut app);
        let hold = app.world().resource::<WeaponsConfig>().stance_hold_secs;

        app.world_mut()
            .get_mut::<ActionIntent>(player)
            .unwrap()
            .aim_held = true;
        app.update();
        assert!(app.world().get::<CombatStance>(player).unwrap().aiming);

        app.world_mut()
            .get_mut::<ActionIntent>(player)
            .unwrap()
            .aim_held = false;
        app.update();
        assert!(
            app.world().get::<CombatStance>(player).unwrap().aiming,
            "hold timer should keep the stance raised"
        );
        step_sim(&mut app, hold + 0.1);
        assert!(!app.world().get::<CombatStance>(player).unwrap().aiming);

        app.world_mut()
            .get_mut::<ActionIntent>(player)
            .unwrap()
            .fire_held = true;
        app.update();
        assert!(app.world().get::<CombatStance>(player).unwrap().aiming);
        app.world_mut()
            .get_mut::<ActionIntent>(player)
            .unwrap()
            .fire_held = false;
        step_sim(&mut app, hold + 0.1);
        assert!(!app.world().get::<CombatStance>(player).unwrap().aiming);

        app.world_mut()
            .get_mut::<WeaponInventory>(player)
            .unwrap()
            .active_mut()
            .unwrap()
            .ammo = 0;
        {
            let mut intent = app.world_mut().get_mut::<ActionIntent>(player).unwrap();
            intent.reload_just_pressed = true;
        }
        app.update();
        assert!(
            app.world()
                .get::<WeaponInventory>(player)
                .unwrap()
                .reload_timer
                .is_some()
        );
        assert!(app.world().get::<CombatStance>(player).unwrap().aiming);

        let reload_secs = app.world().resource::<WeaponsConfig>().weapons[0].reload_secs;
        step_sim(&mut app, reload_secs + 0.1);
        assert!(
            app.world()
                .get::<WeaponInventory>(player)
                .unwrap()
                .reload_timer
                .is_none()
        );
        assert!(
            app.world().get::<CombatStance>(player).unwrap().aiming,
            "stance holds after the reload finishes"
        );
        step_sim(&mut app, hold + 0.1);
        assert!(!app.world().get::<CombatStance>(player).unwrap().aiming);
    }

    #[test]
    fn dead_players_are_not_in_combat_stance() {
        let mut app = test_app();
        app.update();
        let player = player_entity(&mut app);

        app.world_mut()
            .get_mut::<ActionIntent>(player)
            .unwrap()
            .aim_held = true;
        app.update();
        assert!(app.world().get::<CombatStance>(player).unwrap().aiming);

        app.world_mut().get_mut::<Health>(player).unwrap().current = 0.0;
        app.update();
        assert!(!app.world().get::<CombatStance>(player).unwrap().aiming);
    }

    #[test]
    fn swap_cancels_reload_without_partial_refill_and_emits_events() {
        let mut app = test_app();
        app.update();
        let player = player_entity(&mut app);

        app.world_mut()
            .get_mut::<WeaponInventory>(player)
            .unwrap()
            .active_mut()
            .unwrap()
            .ammo = 10;
        {
            let mut intent = app.world_mut().get_mut::<ActionIntent>(player).unwrap();
            intent.reload_just_pressed = true;
        }
        step_sim(&mut app, 0.2);
        let progress = app
            .world()
            .get::<WeaponInventory>(player)
            .unwrap()
            .reload_progress()
            .expect("reload active");
        assert!(progress > 0.0 && progress < 1.0);

        app.world_mut()
            .get_mut::<ActionIntent>(player)
            .unwrap()
            .swap_slot = Some(1);
        app.update();

        let inv = app.world().get::<WeaponInventory>(player).unwrap();
        assert_eq!(inv.active_slot, 1);
        assert!(inv.reload_timer.is_none());
        assert_eq!(inv.slots[0].ammo, 10, "cancelled reload must not refill");

        let cancelled = messages::<ReloadCancelledMessage>(&app);
        assert!(
            cancelled
                .iter()
                .any(|msg| msg.entity == player && msg.reason == ReloadCancelReason::Swapped)
        );
        let swapped = messages::<WeaponSwappedMessage>(&app);
        assert!(
            swapped
                .iter()
                .any(|msg| msg.entity == player && msg.from == 0 && msg.to == 1)
        );
    }

    #[test]
    fn selecting_the_active_slot_keeps_the_reload_running() {
        let mut app = test_app();
        app.update();
        let player = player_entity(&mut app);

        app.world_mut()
            .get_mut::<WeaponInventory>(player)
            .unwrap()
            .active_mut()
            .unwrap()
            .ammo = 10;
        {
            let mut intent = app.world_mut().get_mut::<ActionIntent>(player).unwrap();
            intent.reload_just_pressed = true;
        }
        app.update();
        let before = app
            .world()
            .get::<WeaponInventory>(player)
            .unwrap()
            .reload_timer
            .expect("reload active");

        app.world_mut()
            .get_mut::<ActionIntent>(player)
            .unwrap()
            .swap_slot = Some(0);
        app.update();

        let inv = app.world().get::<WeaponInventory>(player).unwrap();
        let after = inv.reload_timer.expect("reload continues");
        assert!(after < before);
        assert!(messages::<ReloadCancelledMessage>(&app).is_empty());
    }

    #[test]
    fn sprinting_does_not_interrupt_the_reload_timer() {
        let mut app = test_app();
        app.update();
        let player = player_entity(&mut app);

        app.world_mut()
            .get_mut::<WeaponInventory>(player)
            .unwrap()
            .active_mut()
            .unwrap()
            .ammo = 10;
        {
            let mut intent = app.world_mut().get_mut::<ActionIntent>(player).unwrap();
            intent.reload_just_pressed = true;
        }
        app.update();
        let before = app
            .world()
            .get::<WeaponInventory>(player)
            .unwrap()
            .reload_timer
            .expect("reload active");

        {
            let mut intent = app.world_mut().get_mut::<ActionIntent>(player).unwrap();
            intent.sprint = true;
            intent.move_dir = Vec2::new(1.0, 0.0);
        }
        app.world_mut()
            .get_mut::<Stamina>(player)
            .unwrap()
            .is_sprinting = true;
        step_sim(&mut app, 0.2);

        let inv = app.world().get::<WeaponInventory>(player).unwrap();
        let after = inv.reload_timer.expect("reload continues");
        assert!(after < before);
        assert!(messages::<ReloadCancelledMessage>(&app).is_empty());
    }

    #[test]
    fn death_cancels_reload_and_emits_died_reason_once() {
        let mut app = test_app();
        app.update();
        let player = player_entity(&mut app);

        app.world_mut()
            .get_mut::<WeaponInventory>(player)
            .unwrap()
            .active_mut()
            .unwrap()
            .ammo = 10;
        {
            let mut intent = app.world_mut().get_mut::<ActionIntent>(player).unwrap();
            intent.reload_just_pressed = true;
        }
        app.update();
        assert!(
            app.world()
                .get::<WeaponInventory>(player)
                .unwrap()
                .reload_timer
                .is_some()
        );

        app.world_mut()
            .write_message(PlayerDiedMessage { survivor: player });
        app.update();

        assert!(
            app.world()
                .get::<WeaponInventory>(player)
                .unwrap()
                .reload_timer
                .is_none()
        );
        let cancelled = messages::<ReloadCancelledMessage>(&app);
        assert_eq!(
            cancelled
                .iter()
                .filter(|msg| msg.reason == ReloadCancelReason::Died)
                .count(),
            1
        );
    }

    #[test]
    fn reload_lifecycle_emits_started_and_finished() {
        let mut app = test_app();
        app.update();
        let player = player_entity(&mut app);

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
        app.update();

        let started = messages::<ReloadStartedMessage>(&app);
        assert!(
            started
                .iter()
                .any(|msg| msg.entity == player && msg.duration > 0.0)
        );

        app.world_mut()
            .get_mut::<ActionIntent>(player)
            .unwrap()
            .fire_held = false;

        let reload_secs = app.world().resource::<WeaponsConfig>().weapons[0].reload_secs;
        let frames = (reload_secs / (1.0 / 60.0)).ceil() as usize + 5;
        let mut finished_count = 0;
        let mut was_reloading = true;
        for _ in 0..frames {
            app.update();
            let reloading = app
                .world()
                .get::<WeaponInventory>(player)
                .unwrap()
                .reload_timer
                .is_some();
            if was_reloading && !reloading {
                finished_count += messages::<ReloadFinishedMessage>(&app)
                    .iter()
                    .filter(|msg| msg.entity == player)
                    .count();
                was_reloading = false;
            }
        }

        assert_eq!(finished_count, 1);
    }
}
