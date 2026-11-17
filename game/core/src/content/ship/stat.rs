use std::collections::HashMap;

use bevy::prelude::*;

use crate::content::block::{
    BlockOwner,
    feature::{
        storage::{Storage, StorageState},
        thruster::{RotateSpeedComputed, ThrustPowerComputed},
    },
};

/// Single aggregate stats component for a ship.
///
/// Derived from `ShipGrid` by `BlockRegistry::stats_from_grid` on every
/// mutation. Runtime systems update power/thrust/cargo fields from live block
/// state without changing the structural values.
#[derive(Component, Clone, Debug, Reflect)]
pub struct ShipStats {
    pub mass: f32,
    pub max_range: f32,
    pub min_range: f32,
    pub health: f32,
    pub max_health: f32,
    pub power_production: f32,
    pub power_demand: f32,
    pub power_used: f32,
    pub efficiency: f32,
    pub cargo_capacity: u32,
    pub cargo_used: u32,
    pub cargo_items: HashMap<String, u32>,
    pub thrust_power: f32,
    pub rotate_speed: f32,
}

impl Default for ShipStats {
    fn default() -> Self {
        Self {
            mass: 0.0,
            max_range: 0.0,
            min_range: 0.0,
            health: 0.0,
            max_health: 0.0,
            power_production: 0.0,
            power_demand: 0.0,
            power_used: 0.0,
            efficiency: 1.0,
            cargo_capacity: 0,
            cargo_used: 0,
            cargo_items: HashMap::new(),
            thrust_power: 0.0,
            rotate_speed: 0.0,
        }
    }
}

impl ShipStats {
    pub fn is_full(&self) -> bool {
        self.cargo_used >= self.cargo_capacity
    }

    pub fn cargo_free(&self) -> u32 {
        self.cargo_capacity.saturating_sub(self.cargo_used)
    }
}

/// Emitted after a ship grid, stats, or collider changed.
#[derive(Message, Debug, Clone, Copy)]
pub struct ShipChanged {
    pub ship: Entity,
}

/// Aggregate per-block derived components into the ship stats component.
///
/// Structural values (mass, bounds, health, ranges) come from
/// `stats_from_grid`; this system only refreshes values that depend on live
/// block state such as power efficiency and cargo contents.
pub fn update_stats(
    mut ships: Query<&mut ShipStats>,
    blocks: Query<(
        Entity,
        &BlockOwner,
        Option<&ThrustPowerComputed>,
        Option<&RotateSpeedComputed>,
        Option<&Storage>,
        Option<&StorageState>,
    )>,
) {
    for mut stats in &mut ships {
        stats.thrust_power = 0.0;
        stats.rotate_speed = 0.0;
        stats.cargo_items.clear();
        stats.cargo_used = 0;
    }

    for (entity, owner, thrust, rotate, storage, storage_state) in &blocks {
        let Ok(mut stats) = ships.get_mut(owner.ship) else {
            continue;
        };

        if let Some(t) = thrust {
            stats.thrust_power += t.value();
        }
        if let Some(r) = rotate {
            stats.rotate_speed += r.value();
        }

        if let (Some(storage), Some(state)) = (storage, storage_state) {
            for (item, count) in state.iter() {
                stats
                    .cargo_items
                    .entry(item.clone())
                    .and_modify(|c| *c += count)
                    .or_insert(*count);
                stats.cargo_used += *count;
            }
            let _ = (entity, storage);
        }
    }
}
