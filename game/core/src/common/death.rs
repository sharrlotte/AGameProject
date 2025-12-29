use bevy::prelude::*;

#[derive(Component)]
pub struct Death;

pub struct DeathPlugin;

impl Plugin for DeathPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedLast, despawn_dead);
    }
}

pub fn despawn_dead(mut commands: Commands, query: Query<Entity, With<Death>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
