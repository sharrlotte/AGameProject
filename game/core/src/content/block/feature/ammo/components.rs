use std::collections::HashMap;

use bevy::prelude::*;
use bevy::{asset::Handle, image::Image};

use crate::content::block::feature::storage::components::StorageState;
use crate::content::item::components::ItemType;

#[derive(Component, Default, Reflect, Clone, Debug, PartialEq)]
pub struct Ammo {
    capacity: u32,
    crafting: HashMap<ItemType, u32>,
    icon: Option<Handle<Image>>,
}

impl Ammo {
    pub fn insert(&self, commands: &mut Commands, base: Entity) {
        commands
            .entity(base)
            .insert((self.clone(), AmmoState::default()));
    }
}

pub struct AmmoBuilder {
    pub capacity: u32,
    pub crafting: HashMap<ItemType, u32>,
    pub icon: Option<Handle<Image>>,
}

impl AmmoBuilder {
    pub fn build(init: AmmoBuilder) -> Option<Ammo> {
        let AmmoBuilder {
            capacity,
            crafting,
            icon,
        } = init;

        Some(Ammo {
            capacity,
            crafting,
            icon,
        })
    }
}

#[derive(Component, Default, Debug)]
pub struct AmmoState {
    current: u32,
}

impl AmmoState {
    pub fn default() -> Self {
        Self { current: 10 }
    }

    pub fn refill(&mut self, ammo: &Ammo, storage: &mut StorageState) -> bool {
        let mut max_refill = ammo.capacity;

        for (item_type, count) in &ammo.crafting {
            if let Some(amount) = storage.get(item_type) {
                if amount / count < max_refill {
                    max_refill = amount / count;
                }
            }
        }

        if max_refill == 0 {
            return false;
        }

        for (item_type, count) in &ammo.crafting {
            if !storage.take(item_type, max_refill * count) {
                panic!("Storage should have enough items to refill ammo");
            }
        }

        self.current += max_refill;

        true
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
