use bevy::{
    app::{App, Plugin, PreUpdate},
    ecs::{
        component::Component,
        entity::Entity,
        query::With,
        system::{Commands, Query},
    },
};
pub struct DeathPlugin;

impl Plugin for DeathPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, despawn_dead);
    }
}

#[derive(Component)]
pub struct Death;

fn despawn_dead(mut commands: Commands, query: Query<Entity, With<Death>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
