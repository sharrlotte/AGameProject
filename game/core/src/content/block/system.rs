use bevy::{math::VectorSpace, prelude::*};

use crate::{
    common::{damage::message::DamageEvent, death::component::Death},
    constant::{FLASH_OVERLAY_COLOR, LAYER_FLASH_OVERLAY},
    content::block::{
        component::{Block, Health, ShipIdComponent},
        message::BlockDestroyedEvent,
        resource::BlockRegistry,
    },
};

pub fn handle_block_destroy(
    mut commands: Commands,
    mut writer: MessageWriter<BlockDestroyedEvent>,
    mut query: Query<(Entity, &Health, &Block, &ShipIdComponent)>,
) {
    for (entity, health, block, ship_id) in query.iter_mut() {
        if health.health <= 0.0 {
            writer.write(BlockDestroyedEvent {
                block: block.block_type,
                ship_id: ship_id.id,
                block_id: block.id,
            });
            commands.entity(entity).insert(Death);
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
