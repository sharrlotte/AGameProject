use bevy::prelude::*;

#[derive(Clone, PartialEq, Reflect)]
pub struct Texture {
    pub image: Handle<Image>,
    pub offset: Vec2,
    pub z: f32,
    pub animated: bool,
}

impl Texture {
    pub fn insert(&self, commands: &mut Commands, base: Entity, pixel_size: Vec2) {
        commands.entity(base).insert(Sprite {
            image: self.image.clone(),
            custom_size: Some(pixel_size),
            ..default()
        });
    }
}
