use bevy::prelude::*;

use crate::content::block::BlockDef;

#[derive(Clone, PartialEq, Reflect)]
pub struct Texture {
    pub image: Handle<Image>,
    pub offset: Vec2,
    pub z: f32,
    pub animated: bool,
}

impl Texture {
    /// Attach the sprite to the block entity (position comes from the block's
    /// own `Transform`; z-ordering is preserved via the texture's `z`).
    pub fn insert(&self, commands: &mut Commands, base: Entity, def: &BlockDef) {
        commands.entity(base).insert(Sprite {
            image: self.image.clone(),
            custom_size: Some(def.get_pixel_size()),
            ..default()
        });
    }
}
