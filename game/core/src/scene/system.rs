use bevy::prelude::*;

use crate::scene::component::SceneItem;

pub fn clean_scene_items(mut commands: Commands, entities: Query<Entity, With<SceneItem>>) {
    for entity in entities.iter() {
        commands.entity(entity).despawn();
    }
}
