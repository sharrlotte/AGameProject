use crate::content::block::{
    base::BlockOwner,
    stat::{PowerUsageComputed, RotateSpeedComputed, ThrustPowerComputed},
};
use bevy::prelude::*;

#[derive(Component, Clone, Copy, Debug, Reflect, Default)]
pub struct ComputedShipStats {
    pub power_usage: f32,
    pub thrust_power: f32,
    pub rotate_speed: f32,
    pub efficiency: f32,
}

pub fn update_stats(
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
    }

    for (owner, power, thrust, rotate) in &blocks {
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
        }
    }
}
