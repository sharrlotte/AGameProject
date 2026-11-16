use std::collections::HashMap;

use bevy::prelude::*;
use bevy::{asset::Handle, image::Image};

use crate::content::block::feature::storage::StorageState;
use crate::content::item::ItemType;

#[derive(Component, Default, Reflect, Clone, Debug, PartialEq)]
pub struct Ammo {
    capacity: u32,
    crafting: HashMap<ItemType, u32>,
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
    pub crafting: HashMap<ItemType, u32>,
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
    pub fn refill(&mut self, ammo: &Ammo, storage: &mut StorageState) -> bool {
        let room = ammo.capacity.saturating_sub(self.current);

        if room == 0 {
            return true;
        }

        let mut max_refill = room;

        for (item_type, count) in &ammo.crafting {
            if *count == 0 {
                return false;
            }

            let Some(&available) = storage.get(item_type) else {
                return false;
            };

            max_refill = max_refill.min(available / count);
        }

        if max_refill == 0 {
            return false;
        }

        for (item_type, count) in &ammo.crafting {
            let Some(taken) = max_refill.checked_mul(*count) else {
                error!("Ammo refill overflow: {} x {}", max_refill, count);
                return false;
            };

            if !storage.take(item_type, taken) {
                error!("Ammo refill failed to consume {} of {:?}", taken, item_type);
                return false;
            }
        }

        self.current += max_refill;

        self.current >= ammo.capacity
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
