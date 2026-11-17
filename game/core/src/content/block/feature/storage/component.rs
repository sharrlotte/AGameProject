use bevy::{platform::collections::HashMap, prelude::*};


#[derive(Clone, PartialEq, Reflect, Component)]
pub struct Storage {
    capacity: u32,
}

pub struct StorageBuilder {
    pub capacity: u32,
}

impl StorageBuilder {
    pub fn build(init: StorageBuilder) -> Storage {
        let StorageBuilder { capacity } = init;
        Storage { capacity }
    }
}

impl Storage {
    pub fn insert(&self, commands: &mut Commands, base: Entity) {
        commands.entity(base).insert((
            Storage {
                capacity: self.capacity,
            },
            StorageState::default(),
        ));
    }
}

#[derive(Component, Default)]
pub struct StorageState {
    current: u32,
    items: HashMap<String, u32>,
}

impl StorageState {
    pub fn iter(&self) -> impl Iterator<Item = (&String, &u32)> {
        self.items.iter()
    }

    pub fn add(&mut self, storage: &Storage, item_type: String, count: u32) -> bool {
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

    pub fn is_full(&self, storage: &Storage) -> bool {
        self.current >= storage.capacity
    }
}
