pub mod base;
pub mod registry;

use bevy::prelude::*;
use uuid::Uuid;

use crate::{
    components::{damage::DamageEvent, death::Death},
    constant::GRID_SIZE,
    content::block::{
        base::{Block, BlockType, Health, ShipIdComponent},
        registry::{BlockRegistry, register_default_blocks},
    },
};

pub struct BlockPlugin;

impl Plugin for BlockPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<BlockDestroyedEvent>()
            .add_systems(Last, (handle_block_damage_event, handle_flash_overlay))
            .add_systems(Last, handle_block_destroy)
            .add_systems(PreStartup, register_default_blocks);
    }
}

#[derive(Message, Debug, Clone)]
pub struct BlockDestroyedEvent {
    pub block: BlockType,
    pub block_id: Uuid,
    pub ship_id: Entity,
}

fn handle_block_destroy(
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

fn handle_block_damage_event(
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
                    color: Color::srgba(5.0, 5.0, 5.0, 0.8),
                    custom_size: Some(def.shape.size.as_vec2() * GRID_SIZE),
                    ..default()
                },
                FlashOverlay {
                    timer: Timer::from_seconds(0.1, TimerMode::Once),
                },
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
