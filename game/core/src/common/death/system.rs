use bevy::
    ecs::{
        entity::Entity,
        query::With,
        system::{Commands, Query},
    }
;

use crate::common::death::component::Death;

pub fn despawn_dead(mut commands: Commands, query: Query<Entity, With<Death>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
