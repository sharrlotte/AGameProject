use bevy::prelude::*;

use super::*;
use crate::config::GameConfigValue;
use crate::constant::{FLASH_OVERLAY_COLOR, LAYER_DROP, LAYER_FLASH_OVERLAY};
use crate::content::damage::DamageEvent;
use crate::content::item::ItemRegistry;
use crate::content::lifetime::Lifetime;
use crate::content::body::{BlockGrid, BodyMutation};
use crate::resource::SceneState;


pub fn handle_block_destroy(
    mut writer: MessageWriter<BodyMutation>,
    query: Query<(&Health, &BlockRef, &BlockOwner)>,
) {
    for (health, block, owner) in query.iter() {
        if health.health <= 0.0 {
            writer.write(BodyMutation::DestroyBlock {
                body: owner.body,
                block_id: block.id,
            });
        }
    }
}

pub fn handle_block_damage_event(
    mut events: MessageReader<DamageEvent>,
    mut commands: Commands,
    blocks: Query<(&BlockRef, &BlockOwner)>,
    grids: Query<&BlockGrid>,
    registry: Res<BlockRegistry>,
) {
    for event in events.read() {
        let Ok((block, owner)) = blocks.get(event.target) else {
            continue;
        };
        let Ok(grid) = grids.get(owner.body) else {
            continue;
        };
        let Some(grid_block) = grid.block(block.id) else {
            continue;
        };
        let Some(def) = registry.get_symbol(&grid_block.placement.block_type) else {
            continue;
        };

        let overlay = commands
            .spawn((
                Sprite {
                    color: FLASH_OVERLAY_COLOR,
                    custom_size: Some(def.get_base_pixel_size()),
                    ..default()
                },
                FlashOverlay {
                    timer: Timer::from_seconds(0.1, TimerMode::Once),
                },
                Transform::from_translation(Vec2::ZERO.extend(LAYER_FLASH_OVERLAY)),
            ))
            .id();

        commands.entity(event.target).add_child(overlay);
    }
}

pub fn handle_flash_overlay(
    mut commands: Commands,
    mut flash_overlays: Query<(&mut FlashOverlay, Entity)>,
    time: Res<Time>,
) {
    for (mut flash_overlay, entity) in flash_overlays.iter_mut() {
        flash_overlay.timer.tick(time.delta());
        if flash_overlay.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn handle_drop(
    mut commands: Commands,
    mut events: MessageReader<DropEvent>,
    item_registry: Res<ItemRegistry>,
    config: Res<GameConfigValue>,
) {
    for event in events.read() {
        let rarity = match &event.drop {
            Loot::Item(item) => item_registry.get(&item.item_type).map(|f| f.rarity),
        };

        commands.spawn((
            event.drop.clone(),
            DespawnOnExit(SceneState::Game),
            Transform::from_translation(event.position.extend(LAYER_DROP)),
            Sprite {
                color: rarity
                    .map(|f| f.color())
                    .unwrap_or(Color::srgb(5.0, 5.0, 5.0)),
                custom_size: Some(Vec2::new(config.0.drop_size, config.0.drop_size)),
                ..default()
            },
            Lifetime::seconds(config.0.drop_despawn_in_seconds),
        ));
    }
}
