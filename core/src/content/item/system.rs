use super::*;
use bevy::prelude::*;


pub fn register_default_items(mut registry: ResMut<ItemRegistry>) {
    registry.register(ItemDef {
        item_type: "steel".to_string(),
        item_group: ItemGroup::Resource,
        rarity: Rarity::Common,
        name: "steel".to_string(),
        icon: None,
        occupied: 1,
    });
}
