use bevy::prelude::*;
use std::collections::HashMap;

use super::components::*;
use crate::constant::{
    DROP_DESPAWN_IN_SECONDS, DROP_SIZE, FLASH_OVERLAY_COLOR, LAYER_BLOCK, LAYER_DROP,
    LAYER_FLASH_OVERLAY, LAYER_TURRET_OVERLAY,
};
use crate::content::block::component::ComponentPlugin;
use crate::content::block::component::storage::StorageBuilder;
use crate::content::block::component::{
    ammo::AmmoBuilder, components::BlockComponent, sprite::SpriteComponent,
    thruster::ThrusterComponent, turret::TurretComponent,
};
use crate::content::block::radar::RadarPlugin;
use crate::content::damage::DamageEvent;
use crate::content::death::Death;
use crate::content::item::{ItemRegistry, ItemType};
use crate::content::lifetime::Lifetime;
use crate::content::team::Team;

pub struct BlockPlugin;

impl Plugin for BlockPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BlockRegistry>()
            .add_message::<BlockDestroyedEvent>()
            .add_message::<DropEvent>()
            .add_systems(PreStartup, register_default_blocks)
            .add_systems(
                FixedPostUpdate,
                (
                    handle_block_damage_event,
                    handle_flash_overlay,
                    handle_drop,
                    handle_block_destroy,
                ),
            )
            .add_plugins((RadarPlugin, ComponentPlugin));
    }
}

pub fn register_default_blocks(mut registry: ResMut<BlockRegistry>, assets: Res<AssetServer>) {
    let image = |image: String| -> Handle<Image> { assets.load(format!("texture/{image}")) };
    let sound = |sound: String| -> Handle<AudioSource> { assets.load(format!("audio/{sound}")) };

    registry.register(BlockDef {
        name: "Standard Hull".into(),
        block_type: BlockType::Hull,
        block_group: BlockGroup::Base,
        mass: 10.0,
        health: 1000.0,
        shape: BlockShape::default(),
        cost: 5,
        components: vec![BlockComponent::Sprite(SpriteComponent {
            image: image("block/surge-wall-large.png".into()),
            offset: Vec2::ZERO,
            z: LAYER_BLOCK,
            animated: false,
        })],
        drops: vec![DropComponent::Item(ItemDrop {
            item_type: ItemType::Steel,
            amount: 10,
        })],
    });

    registry.register(BlockDef {
        name: "Small Ion Thruster".into(),
        block_type: BlockType::ThrusterSmall,
        block_group: BlockGroup::Base,
        mass: 10.0,
        health: 5000.0,
        shape: BlockShape::default(),
        cost: 50,
        components: vec![
            BlockComponent::Sprite(SpriteComponent {
                image: image("block/thruster.png".into()),
                offset: Vec2::ZERO,
                z: LAYER_BLOCK,
                animated: false,
            }),
            BlockComponent::Thruster(ThrusterComponent::new(200.0, 100.0)),
        ],
        drops: vec![DropComponent::Item(ItemDrop {
            item_type: ItemType::Steel,
            amount: 10,
        })],
    });

    registry.register(BlockDef {
        name: "Core".into(),
        block_type: BlockType::Core,
        block_group: BlockGroup::Core,
        mass: 50.0,
        health: 5000.0,
        shape: BlockShape::filled(3, 3),
        cost: 150,
        components: vec![
            BlockComponent::Sprite(SpriteComponent {
                image: image("block/core-nucleus.png".into()),
                offset: Vec2::ZERO,
                z: LAYER_BLOCK,
                animated: false,
            }),
            BlockComponent::Sprite(SpriteComponent {
                image: image("block/core-nucleus-team.png".into()),
                offset: Vec2::ZERO,
                z: LAYER_TURRET_OVERLAY,
                animated: false,
            }),
            BlockComponent::Thruster(ThrusterComponent::new(100.0, 40.0)),
            BlockComponent::Storage(StorageBuilder::build(StorageBuilder { capacity: 1000 })),
        ],
        drops: vec![DropComponent::Item(ItemDrop {
            item_type: ItemType::Steel,
            amount: 10,
        })],
    });

    registry.register(BlockDef {
        name: "Lazer Turret".into(),
        block_type: BlockType::LazerTurret,
        block_group: BlockGroup::Turret,
        mass: 7.0,
        health: 1600.0,
        shape: BlockShape::filled(1, 1),
        cost: 150,
        components: vec![
            BlockComponent::Sprite(SpriteComponent {
                image: image("block/block-2.png".into()),
                offset: Vec2::ZERO,
                z: LAYER_BLOCK,
                animated: false,
            }),
            BlockComponent::Turret(TurretComponent {
                range: 5000.0,
                turn_speed: 4.0,
                barrels: vec![Vec2::new(0.0, 0.0)],
                strategy: TargetingStrategy::Closest,
                image: image("block/lancer.png".into()),
                offset: Vec2::ZERO,
                z: LAYER_TURRET_OVERLAY,
                animated: false,
                shooting_pattern: ShootPattern::Burst(BurstShootPattern {
                    fire_rate: 3.0,
                    delay: 0.25,
                    amount: 6,
                    projectile: ShootingProjectile::Laser {
                        damage: 40.0,
                        width: 3.0,
                    },
                    shoot_sound: None,
                }),
                ammo: None,
            }),
        ],
        drops: vec![DropComponent::Item(ItemDrop {
            item_type: ItemType::Steel,
            amount: 10,
        })],
    });

    registry.register(BlockDef {
        name: "Scrap Block".into(),
        block_type: BlockType::BlockScrap,
        block_group: BlockGroup::Base,
        mass: 10.0,
        health: 1800.0,
        shape: BlockShape::default(),
        cost: 200,
        components: vec![BlockComponent::Sprite(SpriteComponent {
            image: image("block/block-iron.png".into()),
            offset: Vec2::ZERO,
            z: LAYER_BLOCK,
            animated: false,
        })],
        drops: vec![DropComponent::Item(ItemDrop {
            item_type: ItemType::Steel,
            amount: 10,
        })],
    });

    registry.register(BlockDef {
        name: "Steel Block".into(),
        block_type: BlockType::BlockSteel,
        block_group: BlockGroup::Base,
        mass: 8.0,
        health: 1600.0,
        shape: BlockShape::default(),
        cost: 200,
        components: vec![BlockComponent::Sprite(SpriteComponent {
            image: image("block/block-steel.png".into()),
            offset: Vec2::ZERO,
            z: LAYER_BLOCK,
            animated: false,
        })],
        drops: vec![DropComponent::Item(ItemDrop {
            item_type: ItemType::Steel,
            amount: 10,
        })],
    });

    registry.register(BlockDef {
        name: "Swarmer Turret".into(),
        block_type: BlockType::Swarmer,
        block_group: BlockGroup::Turret,
        mass: 8.0,
        health: 1800.0,
        shape: BlockShape::filled(1, 1),
        cost: 150,
        components: vec![
            BlockComponent::Sprite(SpriteComponent {
                image: image("block/block-2.png".into()),
                offset: Vec2::ZERO,
                z: LAYER_BLOCK,
                animated: false,
            }),
            BlockComponent::Turret(TurretComponent {
                range: 3000.0,
                turn_speed: 4.0,
                strategy: TargetingStrategy::Closest,
                image: image("block/swarmer.png".into()),
                offset: Vec2::ZERO,
                z: LAYER_TURRET_OVERLAY,
                animated: false,
                barrels: vec![Vec2::new(32.0, 0.0), Vec2::new(-32.0, 0.0)],
                shooting_pattern: ShootPattern::Single(SingleShootPattern {
                    fire_rate: 2.0,
                    projectile: ShootingProjectile::Missile {
                        speed: 3000.0,
                        damage: 10.0,
                        lifetime: 10.0,
                        lock_range: 3000.0,
                        blast_damage: 30.0,
                        blast_radius: 32.0,
                        width: 5.0,
                        length: 60.0,
                    },
                }),
                ammo: AmmoBuilder::build(AmmoBuilder {
                    capacity: 1000,
                    crafting: HashMap::from([(ItemType::Steel, 3)]),
                    icon: None,
                }),
            }),
        ],
        drops: vec![DropComponent::Item(ItemDrop {
            item_type: ItemType::Steel,
            amount: 10,
        })],
    });

    registry.register(BlockDef {
        name: "Spectre Turret".into(),
        block_type: BlockType::Spectre,
        block_group: BlockGroup::Turret,
        mass: 16.0,
        health: 3600.0,
        shape: BlockShape::filled(3, 3),
        cost: 150,
        components: vec![
            BlockComponent::Sprite(SpriteComponent {
                image: image("block/block-2.png".into()),
                offset: Vec2::ZERO,
                z: LAYER_BLOCK,
                animated: false,
            }),
            BlockComponent::Turret(TurretComponent {
                range: 5000.0,
                turn_speed: 4.0,
                strategy: TargetingStrategy::Closest,
                image: image("block/spectre.png".into()),
                offset: Vec2::ZERO,
                z: LAYER_TURRET_OVERLAY,
                animated: false,
                barrels: vec![Vec2::new(32.0, 64.0), Vec2::new(-32.0, 64.0)],
                shooting_pattern: ShootPattern::Burst(BurstShootPattern {
                    fire_rate: 0.2,
                    delay: 0.2,
                    amount: 12,
                    projectile: ShootingProjectile::Bullet {
                        speed: 4000.0,
                        damage: 20.0,
                        lifetime: 10.0,
                        piercing: 2,
                        width: 3.0,
                        length: 60.0,
                    },
                    shoot_sound: Some(sound("bullet/shoot-spectre.ogg".into())),
                }),
                ammo: AmmoBuilder::build(AmmoBuilder {
                    capacity: 5000,
                    crafting: HashMap::from([(ItemType::Steel, 1)]),
                    icon: None,
                }),
            }),
        ],
        drops: vec![DropComponent::Item(ItemDrop {
            item_type: ItemType::Steel,
            amount: 10,
        })],
    });

    registry.register(BlockDef {
        name: "Meltdown Turret".into(),
        block_type: BlockType::Meldown,
        block_group: BlockGroup::Turret,
        mass: 20.0,
        health: 3600.0,
        shape: BlockShape::filled(3, 3),
        cost: 150,
        components: vec![
            BlockComponent::Sprite(SpriteComponent {
                image: image("block/block-2.png".into()),
                offset: Vec2::ZERO,
                z: LAYER_BLOCK,
                animated: false,
            }),
            BlockComponent::Turret(TurretComponent {
                range: 3000.0,
                turn_speed: 2.0,
                strategy: TargetingStrategy::Closest,
                image: image("block/meltdown.png".into()),
                offset: Vec2::ZERO,
                z: LAYER_TURRET_OVERLAY,
                animated: false,
                barrels: vec![Vec2::new(0.0, 0.0)],
                shooting_pattern: ShootPattern::LaserBeam(LaserBeamPattern {
                    cooldown: 10.0,
                    duration: 5.0,
                    damage: 1.0,
                    width: 9.0,
                }),
                ammo: None,
            }),
        ],
        drops: vec![DropComponent::Item(ItemDrop {
            item_type: ItemType::Steel,
            amount: 10,
        })],
    });

    registry.register(BlockDef {
        name: "Foreshadow Turret".into(),
        block_type: BlockType::Foreshadow,
        block_group: BlockGroup::Turret,
        mass: 50.0,
        health: 13600.0,
        shape: BlockShape::filled(3, 9),
        cost: 150,
        components: vec![
            BlockComponent::Sprite(SpriteComponent {
                image: image("block/block-2.png".into()),
                offset: Vec2::ZERO,
                z: LAYER_BLOCK,
                animated: false,
            }),
            BlockComponent::Turret(TurretComponent {
                range: 15000.0,
                turn_speed: 0.0,
                strategy: TargetingStrategy::Closest,
                image: image("block/foreshadow.png".into()),
                offset: Vec2::ZERO,
                z: LAYER_TURRET_OVERLAY,
                animated: false,
                barrels: vec![Vec2::new(0.0, 0.0)],
                shooting_pattern: ShootPattern::Single(SingleShootPattern {
                    fire_rate: 10.0,
                    projectile: ShootingProjectile::Bullet {
                        speed: 20000.0,
                        damage: 5000.0,
                        lifetime: 30.0,
                        piercing: 10,
                        width: 12.0,
                        length: 1000.0,
                    },
                }),
                ammo: None,
            }),
        ],
        drops: vec![DropComponent::Item(ItemDrop {
            item_type: ItemType::Steel,
            amount: 10,
        })],
    });

    registry.register(BlockDef {
        name: "Colaris Turret".into(),
        block_type: BlockType::Colaris,
        block_group: BlockGroup::Turret,
        mass: 50.0,
        health: 13600.0,
        shape: BlockShape::filled(1, 1),
        cost: 150,
        components: vec![
            BlockComponent::Sprite(SpriteComponent {
                image: image("block/block-2.png".into()),
                offset: Vec2::ZERO,
                z: LAYER_BLOCK,
                animated: false,
            }),
            BlockComponent::Turret(TurretComponent {
                range: 6000.0,
                turn_speed: 2.0,
                strategy: TargetingStrategy::Closest,
                image: image("block/colaris.png".into()),
                offset: Vec2::ZERO,
                z: LAYER_TURRET_OVERLAY,
                animated: false,
                barrels: vec![Vec2::new(0.0, 0.0)],
                shooting_pattern: ShootPattern::Burst(BurstShootPattern {
                    fire_rate: 2.0,
                    delay: 0.2,
                    amount: 12,
                    shoot_sound: None,
                    projectile: ShootingProjectile::Bullet {
                        speed: 4000.0,
                        damage: 30.0,
                        lifetime: 10.0,
                        piercing: 0,
                        width: 3.0,
                        length: 12.0,
                    },
                }),
                ammo: None,
            }),
        ],
        drops: vec![DropComponent::Item(ItemDrop {
            item_type: ItemType::Steel,
            amount: 10,
        })],
    });

    registry.register(BlockDef {
        name: "Paralax Turret".into(),
        block_type: BlockType::Paralax,
        block_group: BlockGroup::Turret,
        mass: 50.0,
        health: 13600.0,
        shape: BlockShape::filled(1, 1),
        cost: 150,
        components: vec![
            BlockComponent::Sprite(SpriteComponent {
                image: image("block/block-2.png".into()),
                offset: Vec2::ZERO,
                z: LAYER_BLOCK,
                animated: false,
            }),
            BlockComponent::Turret(TurretComponent {
                range: 6000.0,
                turn_speed: 2.0,
                strategy: TargetingStrategy::Closest,
                image: image("block/colaris.png".into()),
                offset: Vec2::ZERO,
                z: LAYER_TURRET_OVERLAY,
                animated: false,
                barrels: vec![Vec2::new(0.0, 0.0)],
                shooting_pattern: ShootPattern::PullingBeam(PullingBeamPattern { width: 3.0 }),
                ammo: None,
            }),
        ],
        drops: vec![DropComponent::Item(ItemDrop {
            item_type: ItemType::Steel,
            amount: 10,
        })],
    });
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
                block_type: block.block_type,
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
) {
    for event in events.read() {
        let rarity = match &event.drop {
            DropComponent::Item(item) => item_registry.get(&item.item_type).map(|f| f.rarity),
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
