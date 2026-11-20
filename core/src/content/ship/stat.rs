use std::collections::HashMap;


use bevy::prelude::*;


use crate::content::block::{
    BlockOwner, BlockRegistry, Health,
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
    pub(crate) cargo_used: u32,
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
    pub fn cargo_used(&self) -> u32 {
        self.cargo_used
    }

    pub fn set_cargo_items(&mut self, items: HashMap<String, u32>) {
        self.cargo_items = items;
        self.cargo_used = self.cargo_items.values().sum::<u32>().min(self.cargo_capacity);
    }

    pub fn add_cargo(&mut self, item: &str, amount: u32) {
        *self.cargo_items.entry(item.to_string()).or_default() += amount;
        self.cargo_used = self.cargo_items.values().sum::<u32>().min(self.cargo_capacity);
    }

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
        if stats.cargo_used() > stats.cargo_capacity {
            stats.cargo_items.clear();
            stats.cargo_used = 0;
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

pub fn sync_ship_health(
    mut ships: Query<&mut ShipStats>,
    blocks: Query<(&BlockOwner, &Health)>,
) {
    for mut stats in &mut ships {
        stats.health = 0.0;
    }
    for (owner, health) in &blocks {
        if let Ok(mut stats) = ships.get_mut(owner.body) {
            stats.health += health.health.max(0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;

    #[test]
    fn ship_health_tracks_partial_block_damage() {
        let mut app = App::new();
        app.add_plugins(bevy::prelude::MinimalPlugins);
        app.add_systems(bevy::prelude::Update, sync_ship_health);

        let ship = app.world_mut().spawn(ShipStats::default()).id();
        app.world_mut().spawn((BlockOwner { body: ship }, Health { health: 3.0, max_health: 10.0 }));
        app.world_mut().spawn((BlockOwner { body: ship }, Health { health: 4.0, max_health: 10.0 }));

        app.update();
        assert_eq!(app.world().get::<ShipStats>(ship).expect("stats").health, 7.0);
    }
}
