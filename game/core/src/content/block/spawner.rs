use std::collections::HashSet;

use bevy::prelude::*;
use bevy_rapier2d::prelude::{
    ActiveEvents, Collider, ColliderMassProperties, CollisionGroups, Sensor,
};

use crate::{
    GRID_SIZE,
    content::{
        block::base::{Block, BlockComponent, BlockDef, GridPosition, Health},
        team::Team,
        turret::base::{AIFireController, PlayerFireController, Turret},
    },
};

pub fn get_block_size(def: &BlockDef) -> Vec2 {
    let size = def.shape.size.as_vec2();
    return size * GRID_SIZE;
}

pub fn get_block_half_size(def: &BlockDef) -> Vec2 {
    return get_block_size(def) * 0.5;
}

pub fn get_block_transform(position: IVec2, half_size: Vec2) -> Transform {
    let anchor_position = Vec3::new(
        position.x as f32 * GRID_SIZE,
        position.y as f32 * GRID_SIZE,
        0.0,
    );

    let position = anchor_position + half_size.extend(0.0);

    return Transform::from_translation(position);
}

pub fn render_block_base(
    commands: &mut Commands,
    parent: Entity,
    def: &BlockDef,
    id: usize,
    position: IVec2,
    team: Team,
) -> Entity {
    let half_size = get_block_half_size(def);
    let transform = get_block_transform(position, half_size);

    let child = commands
        .spawn((
            transform,
            def.block_group,
            team,
            Visibility::Visible,
            GridPosition(position),
            Collider::cuboid(half_size.x, half_size.y),
            CollisionGroups::new(team.get_block_group(), team.get_block_filter()),
            ColliderMassProperties::Mass(0.0),
            Block {
                id,
                rotation: 0,
                block_type: def.block_type,
            },
            Health {
                health: def.health,
                max_health: def.health,
            },
        ))
        .id();

    commands.entity(parent).add_child(child);

    return child;
}

pub fn render_block_component(
    commands: &mut Commands,
    parent: Entity,
    assets: &AssetServer,
    def: &BlockDef,
    component: &BlockComponent,
    team: Team,
) {
    let child = commands
        .spawn((
            Transform::default(),
            GlobalTransform::default(),
            Visibility::Inherited,
        ))
        .id();

    commands.entity(parent).add_child(child);
    commands.entity(child).insert(team);

    match component {
        BlockComponent::Recoil(_recoil_component) => todo!(),
        BlockComponent::HeatEmitter(_heat_emitter_component) => todo!(),
        BlockComponent::EnergyConsumer(_energy_consumer_component) => todo!(),
        BlockComponent::Turret(f) => {
            if team.is_player() {
                commands.entity(child).insert(PlayerFireController);
            } else {
                commands.entity(child).insert(AIFireController);
            }

            commands.entity(child).insert((
                Turret {
                    strategy: f.strategy.clone(),
                    range: f.range,
                    turn_speed: f.turn_speed,
                    fire_all_at_once: f.fire_all_at_once,
                    barrels: f.barrels.clone(),
                    shooting_pattern: f.shooting_pattern.clone(),
                    // state
                    current_barrel_index: 0,
                    potential_targets: HashSet::default(),
                    active: false,
                    target: None,
                    shoot_at: None,
                },
                Sensor,
                Collider::ball(f.range),
                CollisionGroups::new(team.get_radar_group(), team.get_radar_filter()),
                ColliderMassProperties::Mass(0.0),
                ActiveEvents::COLLISION_EVENTS,
            ));
        }
        BlockComponent::Sprite(f) => {
            commands.entity(child).insert((
                Sprite {
                    image: assets.load(&f.sprite),
                    custom_size: Some(get_block_size(def)),
                    color: Color::srgb(1.005, 1.005, 1.005),
                    ..default()
                },
                Transform::from_translation(f.offset.extend(f.z)),
            ));
        }
    }
}
