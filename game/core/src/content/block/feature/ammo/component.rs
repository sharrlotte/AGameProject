use std::collections::HashMap;

use bevy::prelude::*;
use bevy::{asset::Handle, image::Image};

#[derive(Component, Default, Reflect, Clone, Debug, PartialEq)]
pub struct Ammo {
    capacity: u32,
    crafting: HashMap<String, u32>,
    icon: Option<Handle<Image>>,
}

impl Ammo {
    pub fn insert(&self, commands: &mut Commands, base: Entity) {
        commands.entity(base).insert((
            self.clone(),
            AmmoState {
                current: self.capacity,
            },
        ));
    }
}

pub struct AmmoBuilder {
    pub capacity: u32,
    pub crafting: HashMap<String, u32>,
    pub icon: Option<Handle<Image>>,
}

impl AmmoBuilder {
    pub fn build(init: AmmoBuilder) -> Ammo {
        let AmmoBuilder {
            capacity,
            crafting,
            icon,
        } = init;

        Ammo {
            capacity,
            crafting,
            icon,
        }
    }
}

#[derive(Component, Debug)]
pub struct AmmoState {
    current: u32,
}

impl AmmoState {
    pub fn new(current: u32) -> Self {
        Self { current }
    }

    pub fn current(&self) -> u32 {
        self.current
    }

    pub fn set_current(&mut self, current: u32) {
        self.current = current;
    }

    pub fn take(&mut self) -> bool {
        if self.current > 0 {
            self.current -= 1;
            true
        } else {
            false
        }
    }

    pub fn is_empty(&self) -> bool {
        self.current == 0
    }
}
