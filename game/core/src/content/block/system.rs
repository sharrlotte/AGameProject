use bevy::prelude::*;

use super::*;
use crate::constant::{FLASH_OVERLAY_COLOR, LAYER_DROP, LAYER_FLASH_OVERLAY};
use crate::content::block::feature::ComponentPlugin;
use crate::content::block::radar::RadarPlugin;
use crate::content::damage::DamageEvent;
use crate::content::death::Death;
use crate::content::item::ItemRegistry;
use crate::content::lifetime::Lifetime;
use crate::content::team::Team;
use crate::resource::{SceneEntity, SceneState};
use crate::simulation::SimulationSet;
use crate::config::GameConfigValue;

pub struct BlockPlugin;

impl Plugin for BlockPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BlockRegistry>()
            .add_message::<BlockDestroyedEvent>()
            .add_message::<DropEvent>()
            .add_systems(
                FixedPostUpdate,
                (
                    handle_block_damage_event,
                    handle_flash_overlay,
                    handle_drop,
                    handle_block_destroy,
                )
                    .in_set(SimulationSet::Apply),
            )
            .add_plugins((RadarPlugin, ComponentPlugin));
    }
}

pub fn handle_block_destroy(
    mut commands: Commands,
    mut writer: MessageWriter<BlockDestroyedEvent>,
    mut drop_writer: MessageWriter<DropEvent>,
    mut query: Query<(
        Entity,
        &Health,
        &Block,
        &BlockOwner,
        &GlobalTransform,
        &Team,
    )>,
    registry: Res<BlockRegistry>,
) {
    for (entity, health, block, owner, transform, team) in query.iter_mut() {
        if health.health <= 0.0 {
            writer.write(BlockDestroyedEvent {
                block_type: block.block_type.clone(),
                block_id: block.id,
                ship: owner.id,
            });
            commands.entity(entity).insert(Death);

            let Some(def) = registry.get(&block.block_type) else {
                continue;
            };

            if !team.is_player() {
                for drop in def.drops.iter() {
                    drop_writer.write(DropEvent {
                        drop: drop.clone(),
                        position: transform.translation().truncate(),
                    });
                }
            }
        }
    }
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
    config: Res<GameConfigValue>,
) {
    for event in events.read() {
        let rarity = match &event.drop {
            DropComponent::Item(item) => item_registry.get(&item.item_type).map(|f| f.rarity),
        };

        commands.spawn((
            event.drop.clone(),
            SceneEntity(SceneState::Game),
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
