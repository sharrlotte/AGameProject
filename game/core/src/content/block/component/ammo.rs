use std::collections::HashMap;

use bevy::prelude::*;
use bevy::{asset::Handle, image::Image};

use crate::content::block::component::storage::StorageState;
use crate::content::item::base::ItemType;

#[derive(Component, Default, Reflect, Clone, Debug, PartialEq)]
pub struct Ammo {
    capacity: u32,
    crafting: HashMap<ItemType, u32>,
    icon: Option<Handle<Image>>,
}

pub struct AmmoInit {
    pub capacity: u32,
    pub crafting: HashMap<ItemType, u32>,
    pub icon: Option<Handle<Image>>,
}

impl Ammo {
    pub fn new(init: AmmoInit) -> Option<Self> {
        let AmmoInit {
            capacity,
            crafting,
            icon,
        } = init;

        Some(Self {
            capacity,
            crafting,
            icon,
        })
    }
}

#[derive(Component, Default)]
pub struct AmmoState {
    current: u32,
}

impl AmmoState {
    pub fn default() -> Self {
        Self { current: 0 }
    }

    pub fn refill(&mut self, ammo: &Ammo, storage: &mut StorageState) -> bool {
        let mut max_refill = ammo.capacity;

        for (item_type, count) in &ammo.crafting {
            if let Some(amount) = storage.items.get(item_type) {
                if amount / count < max_refill {
                    max_refill = amount / count;
                }
            }
        }

        if max_refill == 0 {
            return false;
        }

        for (item_type, count) in &ammo.crafting {
            if let Some(amount) = storage.items.get_mut(item_type) {
                *amount -= max_refill * count;
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
}
