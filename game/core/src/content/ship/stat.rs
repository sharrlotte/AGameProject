use crate::content::block::{base::BlockOwner, stat::{PowerUsageComputed, RotateSpeedComputed, ThrustPowerComputed}};
use bevy::prelude::*;

#[derive(Component, Clone, Copy, Debug, Reflect, Default)]
pub struct ComputedShipStats {
    pub power_usage: f32,
    pub thrust_power: f32,
    pub rotate_speed: f32,
    /// 0.0 to 1.0 (or higher if buffed/specified, but usually capped at 1.0 for power factor)
    pub efficiency: f32,
}

pub fn update_ship_stats(
    mut ships: Query<&mut ComputedShipStats>,
    blocks: Query<(
        &BlockOwner,
        Option<&PowerUsageComputed>,
        Option<&ThrustPowerComputed>,
        Option<&RotateSpeedComputed>,
    )>,
) {
    for mut stats in &mut ships {
        stats.power_usage = 0.0;
        stats.thrust_power = 0.0;
        stats.rotate_speed = 0.0;
        // efficiency is calculated in a separate system earlier
    }

    for (owner, power, thrust, rotate) in &blocks {
        if let Ok(mut ship_stats) = ships.get_mut(owner.id) {
            if let Some(p) = power {
                ship_stats.power_usage += p.value;
            }
            if let Some(t) = thrust {
                ship_stats.thrust_power += t.value;
            }
            if let Some(r) = rotate {
                ship_stats.rotate_speed += r.value;
            }
        }
    }
}
