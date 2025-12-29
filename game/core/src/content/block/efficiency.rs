use bevy::prelude::*;
use bevy_rapier2d::{prelude::*, rapier::prelude::Ball};
use std::collections::HashMap;

use crate::content::block::base::{BlockOwner, PowerProducer};
use crate::content::block::stat::{BlockEfficiency, EfficiencyProvider, PowerUsageComputed};
use crate::content::ship::stat::ComputedShipStats;

/// Updates the total power usage and production for each ship.
/// Calculates the base efficiency based on supply/demand.
pub fn update_ship_power_totals(
    mut ships: Query<(Entity, &mut ComputedShipStats)>,
    blocks: Query<(
        &BlockOwner,
        Option<&PowerUsageComputed>,
        Option<&PowerProducer>,
    )>,
) {
    // Reset ship stats
    for (_, mut stats) in &mut ships {
        stats.power_usage = 0.0;
        stats.thrust_power = 0.0;
        stats.rotate_speed = 0.0;
        stats.efficiency = 1.0;
    }

    let mut power_map = HashMap::<Entity, (f32, f32)>::new();

    for (owner, usage, producer) in &blocks {
        let entry = power_map.entry(owner.id).or_insert((0.0, 0.0));
        if let Some(u) = usage {
            entry.0 += u.value;
        }
        if let Some(p) = producer {
            entry.1 += p.output;
        }
    }

    for (entity, mut stats) in &mut ships {
        if let Some((usage, production)) = power_map.get(&entity) {
            stats.power_usage = *usage;

            // Calculate Efficiency
            if *usage > 0.0001 {
                if *production >= *usage {
                    stats.efficiency = 1.0;
                } else {
                    stats.efficiency = *production / *usage;
                }
            } else {
                stats.efficiency = 1.0;
            }
        }
    }
}

/// Resets block efficiency to the ship's base efficiency.
pub fn update_block_efficiency_base(
    mut blocks: Query<(&BlockOwner, &mut BlockEfficiency)>,
    ships: Query<&ComputedShipStats>,
) {
    for (owner, mut efficiency) in &mut blocks {
        if let Ok(ship_stats) = ships.get(owner.id) {
            efficiency.base = ship_stats.efficiency;
        } else {
            efficiency.base = 1.0;
        }
        // Reset value to base before applying buffs
        efficiency.value = efficiency.base;
    }
}

/// Applies spatial efficiency buffs.
pub fn update_spatial_efficiency_buffs(
    mut blocks: Query<(Entity, &GlobalTransform, &mut BlockEfficiency)>,
    providers: Query<(&GlobalTransform, &EfficiencyProvider)>,
    rapier_context: ReadRapierContext,
) {
    let rapier = rapier_context.single().expect("Rapier context not found");

    for (transform, provider) in &providers {
        let origin = transform.translation().truncate();
        let shape = Ball::new(provider.radius);
        let filter = QueryFilter::default().exclude_sensors();

        // Fix effect
        rapier.intersect_shape(origin, 0.0, &shape, QueryFilter::new(), |entity| {
            if let Ok((_, _, mut efficiency)) = blocks.get_mut(entity) {
                // Apply buff: Max of current or (Base * Buff)
                let buffed = efficiency.base * provider.value;
                if buffed > efficiency.value {
                    efficiency.value = buffed;
                }
                true
            } else {
                true
            }
        });
    }
}
