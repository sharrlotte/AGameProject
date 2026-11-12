use bevy::prelude::*;
use data::{
    ConfigHotReloadedMessage, ReloadCancelReason, ReloadCancelledMessage, ReloadStartedMessage,
    GameScene, UpgradeModifiers, WeaponFiredMessage, WeaponSwappedMessage,
};
use world::{ActionIntent, AimDirection, PlayerConfig, Stamina, Survivor};

use crate::config::WeaponsConfig;
use crate::projectile::system::spawn_projectile;
use crate::reload::system::start_reload;
use crate::stance::state::CombatStance;

use super::state::{WeaponInventory, WeaponSlotState};

pub fn attach_weapon_inventory(
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

pub fn sync_weapons_hot_reload(
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

/// Ticks weapon cooldowns and handles slot-swap requests (cancelling active reloads).
pub fn handle_weapon_swap(
    time: Res<Time>,
    weapons_cfg: Res<WeaponsConfig>,
    mut cancelled_writer: MessageWriter<ReloadCancelledMessage>,
    mut swapped_writer: MessageWriter<WeaponSwappedMessage>,
    mut shooters: Query<(Entity, &mut ActionIntent, &mut WeaponInventory), With<Survivor>>,
) {
    let dt = time.delta_secs();
    for (entity, mut intent, mut inventory) in &mut shooters {
        inventory.cooldown_timer = (inventory.cooldown_timer - dt).max(0.0);

        let mut target_slot = intent.swap_slot.take();
        if target_slot.is_none() && inventory.slots.len() > 1 {
            if intent.swap_next {
                target_slot = Some((inventory.active_slot + 1) % inventory.slots.len());
            } else if intent.swap_prev {
                target_slot = Some((inventory.active_slot + inventory.slots.len() - 1) % inventory.slots.len());
            }
        }
        intent.swap_next = false;
        intent.swap_prev = false;

        if let Some(target_slot) = target_slot
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
    }
}

/// Consumes firing intent, spawns projectiles, and triggers auto-reload on empty magazines.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn handle_weapon_fire(
    mut commands: Commands,
    scene: Option<Res<State<GameScene>>>,
    weapons_cfg: Res<WeaponsConfig>,
    player_cfg: Res<PlayerConfig>,
    upgrades: Res<UpgradeModifiers>,
    mut fired_writer: MessageWriter<WeaponFiredMessage>,
    mut started_writer: MessageWriter<ReloadStartedMessage>,
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
    let reload_speed = upgrades.reload_speed_multiplier.max(0.1);

    for (entity, transform, aim_dir, stamina, mut intent, mut inventory) in &mut shooters {
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
            start_reload(
                &mut inventory,
                active_spec.reload_secs,
                reload_speed,
                entity,
                &mut started_writer,
            );
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
            let projectile = spawn_projectile(
                &mut commands,
                entity,
                muzzle_pos,
                dir,
                scaled_damage,
                &active_spec,
            );
            if let Some(scene) = scene.as_deref() {
                commands
                    .entity(projectile)
                    .insert(DespawnOnExit(*scene.get()));
            }
        }

        fired_writer.write(WeaponFiredMessage {
            shooter: entity,
            origin: muzzle_pos,
            direction: forward,
        });
    }
}
