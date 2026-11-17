use super::Lifetime;
use crate::simulation::SimulationSet;
use bevy::prelude::*;

pub struct LifetimePlugin;

impl Plugin for LifetimePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, update_lifetime.in_set(SimulationSet::Cleanup));
    }
}

pub fn update_lifetime(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Lifetime)>,
) {
    for (entity, mut lifetime) in query.iter_mut() {
        lifetime.tick(&time);

        if lifetime.timer().is_finished() {
            let Ok(mut commands) = commands.get_entity(entity) else {
                continue;
            };

            commands.despawn();
        }
    }
}
