use bevy::prelude::*;
use bevy_rapier2d::{parry::shape::Ball, plugin::ReadRapierContext, prelude::QueryFilter};

use crate::content::{block::base::BlockOwner, ship::stat::ComputedShipStats};

#[derive(Default, SystemSet, PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub struct EfficiencyUpdateSet;
pub struct EfficiencyPlugin;

impl Plugin for EfficiencyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedFirst,
            (apply_efficiency_buffs, update_block_efficiency_base)
                .chain()
                .in_set(EfficiencyUpdateSet),
        );
    }
}

#[derive(Component, Clone, Copy, Debug, Reflect, Default)]
pub struct EfficiencyEffect {
    pub value: f32,
}

#[derive(Component, Clone, Copy, Debug, Reflect)]
pub struct BlockEfficiency {
    pub value: f32,
    pub base: f32,
}

impl Default for BlockEfficiency {
    fn default() -> Self {
        Self {
            value: 1.0,
            base: 1.0,
        }
    }
}

/// Component that provides efficiency buff to nearby blocks.
#[derive(Component, Clone, Copy, Debug, Reflect, Default, PartialEq)]
pub struct EfficiencyProvider {
    pub radius: f32,
    pub value: f32,
}

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
pub fn apply_efficiency_buffs(
    mut blocks: Query<(Entity, &GlobalTransform, &mut BlockEfficiency)>,
    providers: Query<(&GlobalTransform, &EfficiencyProvider)>,
    rapier_context: ReadRapierContext,
) {
    let rapier = rapier_context.single().expect("Rapier context not found");
    let filter = QueryFilter::default().exclude_sensors();

    for (transform, provider) in &providers {
        let shape = Ball::new(provider.radius);
        let origin = transform.translation().truncate();

        rapier.intersect_shape(origin, 0.0, &shape, filter, |entity| {
            if let Ok((_, _, mut efficiency)) = blocks.get_mut(entity) {
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
