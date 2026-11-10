use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use super::arena::ArenaObstacle;
use super::enemies::Enemy;
use super::input_intent::ActionIntent;
use super::player::{AIM_HEIGHT, AimDirection, Stamina, Survivor};
use super::{DamageMessage, GROUP_ENEMY, GROUP_PROJECTILE, GROUP_WORLD, RunEntity, WeaponFiredMessage};

const MUZZLE_OFFSET: f32 = 0.75;
const SPRINT_SPREAD_MULTIPLIER: f32 = 1.65;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponKind {
    AssaultRifle,
    Shotgun,
}

#[derive(Debug, Clone, Copy)]
pub struct WeaponSpec {
    #[allow(dead_code)]
    pub kind: WeaponKind,
    pub name: &'static str,
    pub full_auto: bool,
    pub fire_interval: f32,
    pub pellets: u8,
    pub damage: f32,
    pub spread_deg: f32,
    pub projectile_speed: f32,
    pub projectile_radius: f32,
    pub projectile_lifetime: f32,
    pub knockback: f32,
    pub mag_size: u32,
    pub reload_secs: f32,
}

pub const ASSAULT_RIFLE_SPEC: WeaponSpec = WeaponSpec {
    kind: WeaponKind::AssaultRifle,
    name: "Assault Rifle",
    full_auto: true,
    fire_interval: 0.10,
    pellets: 1,
    damage: 24.0,
    spread_deg: 3.0,
    projectile_speed: 42.0,
    projectile_radius: 0.10,
    projectile_lifetime: 1.2,
    knockback: 3.5,
    mag_size: 30,
    reload_secs: 1.8,
};

pub const SHOTGUN_SPEC: WeaponSpec = WeaponSpec {
    kind: WeaponKind::Shotgun,
    name: "Shotgun",
    full_auto: false,
    fire_interval: 0.75,
    pellets: 8,
    damage: 14.0,
    spread_deg: 12.0,
    projectile_speed: 34.0,
    projectile_radius: 0.08,
    projectile_lifetime: 0.55,
    knockback: 5.5,
    mag_size: 8,
    reload_secs: 2.2,
};

#[derive(Debug, Clone, Copy)]
pub struct WeaponSlotState {
    pub spec: WeaponSpec,
    pub ammo: u32,
}

impl WeaponSlotState {
    fn new(spec: WeaponSpec) -> Self {
        Self {
            spec,
            ammo: spec.mag_size,
        }
    }
}

/// Holds the survivor's two equipped weapons and reload/fire cooldown state.
#[derive(Component, Debug, Clone)]
pub struct WeaponInventory {
    pub slots: [WeaponSlotState; 2],
    pub active_slot: usize,
    pub cooldown_timer: f32,
    pub reload_timer: Option<f32>,
}

impl Default for WeaponInventory {
    fn default() -> Self {
        Self {
            slots: [
                WeaponSlotState::new(ASSAULT_RIFLE_SPEC),
                WeaponSlotState::new(SHOTGUN_SPEC),
            ],
            active_slot: 0,
            cooldown_timer: 0.0,
            reload_timer: None,
        }
    }
}

impl WeaponInventory {
    pub fn active(&self) -> &WeaponSlotState {
        &self.slots[self.active_slot]
    }

    pub fn active_mut(&mut self) -> &mut WeaponSlotState {
        &mut self.slots[self.active_slot]
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

/// Processes weapon swapping, reloading, and firing from `ActionIntent`.
pub fn handle_weapons(
    mut commands: Commands,
    time: Res<Time>,
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

    for (entity, transform, aim_dir, stamina, mut intent, mut inventory) in &mut shooters {
        inventory.cooldown_timer = (inventory.cooldown_timer - dt).max(0.0);

        if let Some(target_slot) = intent.swap_slot.take() {
            if target_slot < inventory.slots.len() && target_slot != inventory.active_slot {
                inventory.active_slot = target_slot;
                // Cancel any active reload when swapping weapons.
                inventory.reload_timer = None;
                inventory.cooldown_timer = inventory.cooldown_timer.min(0.12);
            }
        }

        if let Some(remaining) = inventory.reload_timer.as_mut() {
            *remaining -= dt;
            if *remaining <= 0.0 {
                let mag_size = inventory.active().spec.mag_size;
                inventory.active_mut().ammo = mag_size;
                inventory.reload_timer = None;
            }
        }

        let want_reload = intent.reload_just_pressed;
        intent.reload_just_pressed = false;
        if want_reload && inventory.reload_timer.is_none() {
            let active = inventory.active();
            if active.ammo < active.spec.mag_size {
                inventory.reload_timer = Some(active.spec.reload_secs);
            }
        }

        let fire_just_pressed = intent.fire_just_pressed;
        intent.fire_just_pressed = false;

        if inventory.reload_timer.is_some() || inventory.cooldown_timer > 0.0 {
            continue;
        }

        let spec = inventory.active().spec;
        let triggered = if spec.full_auto {
            intent.fire_held
        } else {
            fire_just_pressed
        };

        if !triggered {
            continue;
        }

        if inventory.active().ammo == 0 {
            // Auto-reload when pulling the trigger on an empty magazine.
            inventory.reload_timer = Some(spec.reload_secs);
            continue;
        }

        inventory.active_mut().ammo -= 1;
        inventory.cooldown_timer = spec.fire_interval;

        let forward = if aim_dir.0.length_squared() > 1e-4 {
            aim_dir.0.normalize()
        } else {
            Vec3::NEG_Z
        };
        let muzzle_pos = Vec3::new(transform.translation.x, AIM_HEIGHT, transform.translation.z)
            + forward * MUZZLE_OFFSET;

        let spread_mult = if stamina.is_sprinting {
            SPRINT_SPREAD_MULTIPLIER
        } else {
            1.0
        };
        let half_spread_rad = (spec.spread_deg * spread_mult).to_radians() * 0.5;

        for _ in 0..spec.pellets {
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
                    damage: spec.damage,
                    knockback: spec.knockback,
                    direction: dir,
                    radius: spec.projectile_radius,
                    lifetime: spec.projectile_lifetime,
                    hit: false,
                },
                RigidBody::Dynamic,
                GravityScale(0.0),
                Ccd::enabled(),
                Sensor,
                ActiveEvents::COLLISION_EVENTS,
                Collider::ball(spec.projectile_radius),
                Velocity::linear(dir * spec.projectile_speed),
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
    enemies: Query<(), With<Enemy>>,
    obstacles: Query<(), With<ArenaObstacle>>,
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
    use crate::sim::player::spawn_player;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
                Duration::from_secs_f32(1.0 / 60.0),
            ))
            .add_message::<WeaponFiredMessage>()
            .add_systems(Startup, spawn_player)
            .add_systems(Update, handle_weapons);
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

        // Fire Assault Rifle once
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
            app.world().get::<WeaponInventory>(player).unwrap().active().ammo,
            ASSAULT_RIFLE_SPEC.mag_size - 1
        );

        // Start reload, then swap to Shotgun -> reload must cancel
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

        // Fire Shotgun -> spawns 8 pellets
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
        assert_eq!(total_projs, 1 + SHOTGUN_SPEC.pellets as usize);
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

        app.world_mut()
            .get_mut::<WeaponInventory>(player)
            .unwrap()
            .active_mut()
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

        // Advance past full reload duration
        app.world_mut()
            .get_mut::<ActionIntent>(player)
            .unwrap()
            .fire_held = false;
        step_sim(&mut app, ASSAULT_RIFLE_SPEC.reload_secs + 0.1);

        let inv = app.world().get::<WeaponInventory>(player).unwrap();
        assert!(inv.reload_timer.is_none());
        assert_eq!(inv.active().ammo, ASSAULT_RIFLE_SPEC.mag_size);
    }
}
