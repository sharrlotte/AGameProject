use bevy::prelude::*;
use bevy_rapier2d::{parry::shape::Ball, plugin::ReadRapierContext, prelude::QueryFilter};

use super::components::*;
use crate::content::{block::components::BlockOwner, ship::stat::ComputedShipStats};

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

pub fn update_block_efficiency_base(
    mut blocks: Query<(&BlockOwner, &mut BlockEfficiency)>,
    ships: Query<&ComputedShipStats>,
) {
    for (owner, mut efficiency) in &mut blocks {
        // Reset value to base before applying buffs
        if let Ok(ship_stats) = ships.get(owner.id) {
            efficiency.set(ship_stats.efficiency);
        } else {
            efficiency.set(1.0);
        }
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
                efficiency.apply(provider.value);

                true
            } else {
                true
            }
        });
    }
}
