use bevy::prelude::*;

use crate::{
    common::{
        damage::message::DamageEvent, death::component::Death, lifetime::component::Lifetime,
    },
    constant::{
        DROP_DESPAWN_IN_SECONDS, DROP_SIZE, FLASH_OVERLAY_COLOR, LAYER_DROP, LAYER_FLASH_OVERLAY,
    },
    content::{
        block::{
            component::{Block, Health, ShipIdComponent},
            message::BlockDestroyedEvent,
            resource::{BlockRegistry, Drop, DropEvent},
        },
        item::resource::ItemRegistry,
    },
};

pub fn handle_block_destroy(
    mut commands: Commands,
    mut writer: MessageWriter<BlockDestroyedEvent>,
    mut drop_writer: MessageWriter<DropEvent>,
    mut query: Query<(Entity, &Health, &Block, &ShipIdComponent, &GlobalTransform)>,
    registry: Res<BlockRegistry>,
) {
    for (entity, health, block, ship_id, transform) in query.iter_mut() {
        if health.health <= 0.0 {
            writer.write(BlockDestroyedEvent {
                block_type: block.block_type,
                block_id: block.id,
                ship: ship_id.id,
            });
            commands.entity(entity).insert(Death);

            let Some(def) = registry.get(&block.block_type) else {
                continue;
            };

            for drop in def.drops.iter() {
                drop_writer.write(DropEvent {
                    drop: drop.clone(),
                    position: transform.translation().truncate(),
                });
            }
        }
    }
}

#[derive(Component, Debug)]
pub struct FlashOverlay {
    timer: Timer,
}

pub fn handle_block_damage_event(
    mut events: MessageReader<DamageEvent>,
    mut commands: Commands,
    blocks: Query<&Block>,
    registry: Res<BlockRegistry>,
) {
    for event in events.read() {
        let Ok(block) = blocks.get(event.target) else {
            continue;
        };

        let Some(def) = registry.get(&block.block_type) else {
            continue;
        };

        let overlay = commands
            .spawn((
                Sprite {
                    color: FLASH_OVERLAY_COLOR,
                    custom_size: Some(def.get_pixel_size()),
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
) {
    for event in events.read() {
        let rarity = match &event.drop {
            Drop::Item(item) => item_registry.get(&item.item_type).map(|f| f.rarity),
        };

        commands.spawn((
            event.drop.clone(),
            Transform::from_translation(event.position.extend(LAYER_DROP)),
            Sprite {
                color: rarity
                    .map(|f| f.color())
                    .unwrap_or(Color::srgb(5.0, 5.0, 5.0)),
                custom_size: Some(Vec2::new(DROP_SIZE, DROP_SIZE)),
                ..default()
            },
            Lifetime::seconds(DROP_DESPAWN_IN_SECONDS),
        ));
    }
}
