use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use super::component::{Capability, Readiness};
use super::config::AiConfig;
use crate::content::block::feature::ammo::AmmoState;
use crate::content::block::feature::turret::{TurretCharge, TurretPower, TurretState};
use crate::content::block::{
    BlockOwner, Disabled, LaserBeamPatternShootingState, LaserBeamPatternState,
    PullingBeamPattern,
};
use crate::content::ship::ShipStats;

pub fn update_readiness(
    mut ships: Query<(Entity, &ShipStats, &Capability, &mut Readiness)>,
    blocks: Query<(
        &BlockOwner,
        Option<&AmmoState>,
        Option<(&TurretPower, &TurretCharge)>,
        Option<&LaserBeamPatternState>,
        Option<&PullingBeamPattern>,
        Option<&TurretState>,
    )>,
    mut ammo_totals: Local<HashMap<Entity, (u32, u32)>>,
    mut charge_totals: Local<HashMap<Entity, (f32, f32)>>,
    mut draining: Local<HashSet<Entity>>,
) {
    ammo_totals.clear();
    charge_totals.clear();
    draining.clear();

    for (owner, ammo_state, turret_charge, laser, pulling, state) in &blocks {
        if let Some(ammo_state) = ammo_state {
            let entry = ammo_totals.entry(owner.body).or_default();
            entry.0 += ammo_state.current();
            entry.1 += 1;
        }
        if let Some((TurretPower::Charge { capacity, .. }, charge)) = turret_charge {
            let entry = charge_totals.entry(owner.body).or_default();
            entry.0 += charge.value;
            entry.1 += capacity;
        }
        let beam_on = laser.is_some_and(|l| {
            matches!(l.shooting, LaserBeamPatternShootingState::Shooting { .. })
        }) || (pulling.is_some() && state.is_some_and(|s| s.active()));
        if beam_on {
            draining.insert(owner.body);
        }
    }

    for (entity, stats, cap, mut readiness) in &mut ships {
        let hp_ratio = if stats.max_health > 0.0 {
            (stats.health / stats.max_health).clamp(0.0, 1.0)
        } else {
            1.0
        };

        let battery_ratio = if stats.power_capacity > 0.0 {
            (stats.power_stored / stats.power_capacity).clamp(0.0, 1.0)
        } else {
            1.0
        };

        let charge_ratio = match charge_totals.get(&entity) {
            Some(&(cur, max)) if max > 0.0 => (cur / max).clamp(0.0, 1.0),
            _ => 1.0,
        };

        let energy_ratio = if cap.uses_energy_weapons {
            (stats.power_supply * 0.5 + battery_ratio * 0.25 + charge_ratio * 0.25).clamp(0.0, 1.0)
        } else {
            (stats.power_supply * 0.7 + battery_ratio * 0.3).clamp(0.0, 1.0)
        };

        let steel = stats.cargo_items.get("steel").copied().unwrap_or(0);
        let ammo_ratio = match ammo_totals.get(&entity) {
            Some(&(current, count)) if count > 0 => {
                if current > 0 || steel > 0 {
                    ((current as f32 + steel as f32 * 50.0) / (count as f32 * 500.0)).clamp(0.0, 1.0)
                } else {
                    0.0
                }
            }
            _ => 1.0,
        };

        let is_support_role = matches!(cap.role, super::component::Role::Support | super::component::Role::Escort);
        let disarmed = (!is_support_role && cap.offensive_dps <= 0.0)
            || (!is_support_role
                && cap.uses_ammo_weapons
                && !cap.uses_energy_weapons
                && ammo_ratio <= 0.0);

        *readiness = Readiness {
            hp_ratio,
            energy_ratio,
            ammo_ratio,
            disarmed,
            firing_drain_beam: draining.contains(&entity),
            power_supply: stats.power_supply,
            battery_ratio,
        };
    }
}

#[allow(clippy::too_many_arguments)]
pub fn update_booster_shedding(
    commands: &mut Commands,
    ship_entity: Entity,
    skill: u8,
    cap: &Capability,
    readiness: &Readiness,
    cfg: &AiConfig,
    boosters_by_ship: &HashMap<Entity, Vec<Entity>>,
    shedding_ships: &HashSet<Entity>,
) {
    let Some(list) = boosters_by_ship.get(&ship_entity) else {
        return;
    };
    let shedding = shedding_ships.contains(&ship_entity);

    if skill < cfg.unlocks.power_management_level || cap.booster_count == 0 {
        if shedding {
            for &booster in list {
                commands.entity(booster).remove::<Disabled>();
            }
        }
        return;
    }

    if !shedding
        && readiness.power_supply < cfg.booster_shed_supply_threshold
        && readiness.battery_ratio < cfg.booster_shed_battery_threshold
    {
        for &booster in list {
            commands.entity(booster).insert(Disabled);
        }
    } else if shedding
        && (readiness.battery_ratio >= cfg.booster_restore_battery_threshold
            || readiness.power_supply >= 0.99)
    {
        for &booster in list {
            commands.entity(booster).remove::<Disabled>();
        }
    }
}
