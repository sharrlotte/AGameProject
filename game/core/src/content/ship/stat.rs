use std::collections::HashMap;

use crate::content::{
    block::{
        components::BlockOwner,
        component::{
            energy::PowerUsageComputed,
            storage::{Storage, StorageState},
            thruster::{RotateSpeedComputed, ThrustPowerComputed},
        },
    },
    item::ItemType,
};
use bevy::prelude::*;

#[derive(Component, Clone, Debug, Reflect, Default)]
pub struct ComputedShipStats {
    pub power_usage: f32,
    pub thrust_power: f32,
    pub rotate_speed: f32,
    pub efficiency: f32,
    pub is_full: bool,
    pub items: HashMap<ItemType, u32>,
    pub storages: Vec<Entity>,
}

pub fn update_stats(
    mut ships: Query<&mut ComputedShipStats>,
    blocks: Query<(
        Entity,
        &BlockOwner,
        Option<&PowerUsageComputed>,
        Option<&ThrustPowerComputed>,
        Option<&RotateSpeedComputed>,
        Option<&Storage>,
        Option<&StorageState>,
    )>,
) {
    for mut stats in &mut ships {
        stats.power_usage = 0.0;
        stats.thrust_power = 0.0;
        stats.rotate_speed = 0.0;
        stats.is_full = true;
        stats.items.clear();
        stats.storages.clear();
    }

    for (entity,owner, power, thrust, rotate, storage, storage_state) in &blocks {
        if let Ok(mut stats) = ships.get_mut(owner.id) {
            if let Some(p) = power {
                stats.power_usage += p.value;
            }
            if let Some(t) = thrust {
                stats.thrust_power += t.value;
            }
            if let Some(r) = rotate {
                stats.rotate_speed += r.value;
            }

            if let (Some(s), Some(state)) = (storage, storage_state) {
                if !state.is_full(s) {
                    stats.is_full = false;
                }

                for (item, count) in &state.items {
                    stats
                        .items
                        .entry(*item)
                        .and_modify(|c| *c += count)
                        .or_insert(*count);
                }

                stats.storages.push(entity);
            }
        }
    }
}
