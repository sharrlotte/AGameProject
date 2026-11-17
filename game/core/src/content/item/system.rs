use bevy::prelude::*;
use super::*;

pub struct ItemPlugin;

impl Plugin for ItemPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ItemRegistry>()
            .add_systems(PreStartup, register_default_items);
    }
}

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
