use bevy::{platform::collections::HashMap, prelude::*};

use crate::content::item::base::ItemType;

#[derive(Clone, PartialEq, Reflect)]
pub struct StorageComponent {
    pub capacity: u32,
}

impl StorageComponent {
    pub fn spawn(&self, commands: &mut Commands, base: Entity) {
        commands.entity(base).insert((
            Storage {
                capacity: self.capacity,
            },
            StorageState::default(),
        ));
    }
}

#[derive(Component)]
pub struct Storage {
    pub capacity: u32,
}

#[derive(Component, Default)]
pub struct StorageState {
    current: u32,
    pub items: HashMap<ItemType, u32>,
}

impl StorageState {
    pub fn add(&mut self, storage: &Storage, item_type: ItemType, count: u32) -> bool {
        if self.current + count > storage.capacity {
            return false;
        }

        self.items
            .entry(item_type)
            .and_modify(|c| *c += count)
            .or_insert(count);

        self.current += count;

        true
    }

    pub fn take(&mut self, item_type: ItemType, count: u32) -> bool {
        if let Some(c) = self.items.get_mut(&item_type) {
            if *c < count {
                return false;
            }
            *c -= count;
            self.current -= count;
            return true;
        }

        false
    }

    pub fn is_full(&self, storage: &Storage) -> bool {
        if self.current > storage.capacity {
            panic!(
                "Storage current count {} is greater than capacity {}",
                self.current, storage.capacity
            );
        }

        self.current == storage.capacity
    }
}
