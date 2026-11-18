use std::collections::HashMap;


use bevy::prelude::*;


use crate::content::block::{
    BlockOwner, BlockRegistry,
    feature::thruster::{RotateSpeedComputed, ThrustPowerComputed},
};

use crate::content::body::BlockGrid;


#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct ShipStats {
    pub mass: f32,
    pub max_range: f32,
    pub min_range: f32,
    pub health: f32,
    pub max_health: f32,
    pub power_production: f32,
    pub power_demand: f32,
    pub power_supply: f32,
    pub power_stored: f32,
    pub power_capacity: f32,
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
            power_supply: 1.0,
            power_stored: 0.0,
            power_capacity: 0.0,
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
}


#[derive(Message, Debug, Clone, Copy)]
pub struct ShipChanged {
    pub ship: Entity,
}


pub fn rebuild_cargo_capacity(
    mut events: MessageReader<ShipChanged>,
    mut ships: Query<(&mut ShipStats, &BlockGrid)>,
    registry: Res<BlockRegistry>,
) {
    for event in events.read() {
        let Ok((mut stats, grid)) = ships.get_mut(event.ship) else {
            continue;
        };

        let Some(derived) = registry.stats_from_grid(grid) else {
            continue;
        };
        
        stats.cargo_capacity = derived.cargo_capacity;
        if stats.cargo_used > stats.cargo_capacity {
            stats.cargo_used = stats.cargo_capacity;
            stats.cargo_items.clear();
        }
    }
}


pub fn update_stats(
    mut ships: Query<&mut ShipStats>,
    blocks: Query<(
        Entity,
        &BlockOwner,
        Option<&ThrustPowerComputed>,
        Option<&RotateSpeedComputed>,
    )>,
) {
    for mut stats in &mut ships {
        stats.thrust_power = 0.0;
        stats.rotate_speed = 0.0;
    }

    for (_, owner, thrust, rotate) in &blocks {
        let Ok(mut stats) = ships.get_mut(owner.body) else {
            continue;
        };

        if let Some(t) = thrust {
            stats.thrust_power += t.value();
        }
        if let Some(r) = rotate {
            stats.rotate_speed += r.value();
        }
    }
}
