use bevy::prelude::*;
use bevy_rapier2d::{parry::shape::Ball, plugin::ReadRapierContext, prelude::QueryFilter};

use super::*;
use crate::content::{block::BlockOwner, ship::stat::ShipStats, team::Team};
use crate::simulation::SimulationSet;

pub struct EfficiencyPlugin;

impl Plugin for EfficiencyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedFirst,
            (update_block_efficiency_base, apply_efficiency_buffs)
                .chain()
                .in_set(EfficiencyUpdateSet)
                .in_set(SimulationSet::Stats),
        );
    }
}

pub fn update_block_efficiency_base(
    mut blocks: Query<(&BlockOwner, &mut BlockEfficiency)>,
    ships: Query<&ShipStats>,
) {
    for (owner, mut efficiency) in &mut blocks {
        // Reset value to base before applying buffs
        if let Ok(ship_stats) = ships.get(owner.ship) {
            efficiency.set(ship_stats.efficiency);
        } else {
            efficiency.set(1.0);
        }
    }
}

/// Applies spatial efficiency buffs.
pub fn apply_efficiency_buffs(
    mut blocks: Query<(Entity, &GlobalTransform, &mut BlockEfficiency, &Team)>,
    providers: Query<(&GlobalTransform, &EfficiencyProvider, &Team)>,
    rapier_context: ReadRapierContext,
) {
    let Ok(rapier) = rapier_context.single() else {
        return;
    };
    let filter = QueryFilter::default();

    for (transform, provider, provider_team) in &providers {
        let shape = Ball::new(provider.radius());
        let origin = transform.translation().truncate();

        rapier.intersect_shape(origin, 0.0, &shape, filter, |entity, _collider| {
            if let Ok((_, _, mut efficiency, team)) = blocks.get_mut(entity) {
                if team.id == provider_team.id {
                    efficiency.apply(provider.value());
                }

                true
            } else {
                true
            }
        });
    }
}
