use std::collections::HashMap;

use bevy::prelude::*;
use bevy::{asset::Handle, image::Image};

use crate::content::item::base::ItemType;

#[derive(Component, Default)]
pub struct Ammo {
    capacity: u32,
    crafting: HashMap<ItemType, u32>,
    icon: Option<Handle<Image>>,
}

#[derive(Component, Default)]
pub struct AmmoState {
    current: u32,
}
