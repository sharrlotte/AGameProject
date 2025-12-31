use bevy::prelude::*;

use crate::content::block::components::BlockDef;

#[derive(Clone, PartialEq, Reflect)]
pub struct SpriteComponent {
    pub image: Handle<Image>,
    pub offset: Vec2,
    pub z: f32,
    pub animated: bool,
}

impl SpriteComponent {
    pub fn spawn(&self, commands: &mut Commands, base: Entity, def: &BlockDef) {
        commands.entity(base).insert((
            Sprite {
                image: self.image.clone(),
                custom_size: Some(def.get_pixel_size()),
                ..default()
            },
            Transform::from_translation(self.offset.extend(self.z)),
        ));
    }
}
