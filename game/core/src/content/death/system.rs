use super::Death;
use crate::simulation::SimulationSet;
use bevy::prelude::*;

pub struct DeathPlugin;

impl Plugin for DeathPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedLast, despawn_dead.in_set(SimulationSet::Cleanup));
    }
}

pub fn despawn_dead(mut commands: Commands, query: Query<Entity, With<Death>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
