use bevy::prelude::*;
use std::collections::HashMap;

pub struct ItemPlugin;

impl Plugin for ItemPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ItemRegistry>()
            .add_systems(PreStartup, register_default_items);
    }
}

#[derive(Debug, Reflect, Hash, Eq, PartialEq, Copy, Clone)]
pub enum ItemType {
    Steel,
}

#[derive(Debug, Reflect, Copy, Clone)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

impl Rarity {
    pub fn color(&self) -> Color {
        match self {
            Rarity::Common => Color::srgb(5.0, 5.0, 5.0),
            Rarity::Uncommon => Color::srgb(2.5, 5.0, 2.5),
            Rarity::Rare => Color::srgb(2.5, 2.5, 5.0),
            Rarity::Epic => Color::srgb(5.0, 2.5, 5.0),
            Rarity::Legendary => Color::srgb(5.0, 5.0, 2.5),
        }
    }
}

#[derive(Debug, Reflect, Hash, Eq, PartialEq, Copy, Clone, Ord, PartialOrd)]
pub enum ItemGroup {
    Resource,
}

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct ItemRegistry {
    map: HashMap<ItemType, ItemDef>,
}

#[derive(Debug, Reflect)]
pub struct ItemDef {
    pub item_type: ItemType,
    pub item_group: ItemGroup,
    pub rarity: Rarity,
    pub name: String,
    pub icon: Option<Handle<Image>>,
    pub occupied: usize,
}

impl ItemRegistry {
    fn register(&mut self, def: ItemDef) {
        if self.map.contains_key(&def.item_type) {
            warn!("Item type {:#?} is already registered.", def.item_type);
            return;
        }

        self.map.insert(def.item_type, def);
    }

    pub fn get(&self, item_type: &ItemType) -> Option<&ItemDef> {
        self.map.get(item_type)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ItemType, &ItemDef)> {
        let mut value = self.map.iter().collect::<Vec<_>>();

        value.sort_by(|a, b| a.1.item_group.cmp(&b.1.item_group));

        return value.into_iter();
    }
}

pub fn register_default_items(mut registry: ResMut<ItemRegistry>) {
    registry.register(ItemDef {
        item_type: ItemType::Steel,
        item_group: ItemGroup::Resource,
        rarity: Rarity::Common,
        name: "steal".to_string(),
        icon: None,
        occupied: 1,
    });
}
