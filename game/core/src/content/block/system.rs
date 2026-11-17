use bevy::prelude::*;

use super::*;
use crate::constant::{FLASH_OVERLAY_COLOR, LAYER_DROP, LAYER_FLASH_OVERLAY};
use crate::content::block::feature::ComponentPlugin;
use crate::content::block::radar::RadarPlugin;
use crate::content::damage::DamageEvent;
use crate::content::death::Death;
use crate::content::item::ItemRegistry;
use crate::content::lifetime::Lifetime;
use crate::content::ship::{ShipGrid, ShipMutation};
use crate::resource::{SceneState};
use crate::simulation::SimulationSet;
use crate::config::GameConfigValue;

pub struct BlockPlugin;

impl Plugin for BlockPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BlockRegistry>()
            .add_message::<DropEvent>()
            .add_systems(
                FixedPostUpdate,
                (
                    handle_block_damage_event,
                    handle_flash_overlay,
                    handle_block_destroy,
                )
                    .in_set(SimulationSet::Death),
            )
            .add_systems(
                FixedPostUpdate,
                handle_drop
                    .after(crate::content::ship::mutation::apply_ship_mutations)
                    .in_set(SimulationSet::Death),
            )
            .add_plugins((RadarPlugin, ComponentPlugin));
    }
}

pub fn handle_block_destroy(
    mut commands: Commands,
    mut writer: MessageWriter<ShipMutation>,
    mut query: Query<(
        Entity,
        &Health,
        &BlockRef,
        &BlockOwner,
    )>,
) {
    for (entity, health, block, owner) in query.iter_mut() {
        if health.health <= 0.0 {
            writer.write(ShipMutation::DestroyBlock {
                ship: owner.ship,
                block_id: block.id,
            });
            commands.entity(entity).insert(Death);
        }
    }
}

pub fn handle_block_damage_event(
    mut events: MessageReader<DamageEvent>,
    mut commands: Commands,
    blocks: Query<(&BlockRef, &BlockOwner)>,
    grids: Query<&ShipGrid>,
    registry: Res<BlockRegistry>,
) {
    for event in events.read() {
        let Ok((block, owner)) = blocks.get(event.target) else {
            continue;
        };
        let Ok(grid) = grids.get(owner.ship) else {
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
