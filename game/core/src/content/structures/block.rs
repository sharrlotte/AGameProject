use std::collections::HashSet;

use bevy::prelude::*;
use bevy_rapier2d::prelude::{
    ActiveEvents, Collider, ColliderMassProperties, CollisionGroups, Sensor,
};

use crate::{
    GRID_SIZE,
    content::{
        structures::{components::Health, ship::ShipBlockHitbox, types::*},
        team::Team,
        turret::types::{AIFireController, PlayerFireController, TurretController},
    },
};

pub fn get_block_size(def: &BlockDef) -> Vec2 {
    let size = def.shape.size.as_vec2();
    return size * GRID_SIZE;
}

pub fn get_block_half_size(def: &BlockDef) -> Vec2 {
    return get_block_size(def) * 0.5;
}

pub fn get_block_transform(grid_pos: IVec2, half_size: Vec2) -> Transform {
    let anchor_pos = Vec3::new(
        grid_pos.x as f32 * GRID_SIZE,
        grid_pos.y as f32 * GRID_SIZE,
        0.0,
    );

    let pos = anchor_pos + half_size.extend(0.0);

    return Transform::from_translation(pos);
}

pub fn render_block_base(
    commands: &mut Commands,
    parent: Entity,
    def: &BlockDef,
    id: usize,
    grid_pos: IVec2,
    team: Team,
) -> Entity {
    let half_size = get_block_half_size(def);
    let transform = get_block_transform(grid_pos, half_size);

    let child = commands
        .spawn((
            transform,
            def.block_group,
            team,
            Visibility::Visible,
            GridPosition(grid_pos),
            PlacedBlock,
            ShipBlockHitbox,
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

pub fn render_block_layer(
    commands: &mut Commands,
    parent: Entity,
    layer: &BlockLayerDef,
) -> Entity {
    let child = commands
        .spawn((
            BlockLayer {
                kind: layer.kind.clone(),
            },
            Transform::default(),
            GlobalTransform::default(),
            Visibility::Inherited,
        ))
        .id();

    commands.entity(parent).add_child(child);

    return child;
}

pub fn render_block_layer_feature(
    commands: &mut Commands,
    parent: Entity,
    assets: &AssetServer,
    def: &BlockDef,
    feature: &LayerFeature,
    team: Team,
) {
    if team.is_player() {
        commands.entity(parent).insert(PlayerFireController);
    } else {
        commands.entity(parent).insert(AIFireController);
    }

    commands.entity(parent).insert(team);

    match feature {
        LayerFeature::Recoil(_recoil_feature) => todo!(),
        LayerFeature::HeatEmitter(_heat_emitter_feature) => todo!(),
        LayerFeature::EnergyConsumer(_energy_consumer_feature) => todo!(),
        LayerFeature::Turret(f) => {
            commands.entity(parent).insert((
                TurretController {
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
                Collider::ball(f.range),
                CollisionGroups::new(team.get_radar_group(), team.get_radar_filter()),
                Sensor,
                ColliderMassProperties::Mass(0.0),
                ActiveEvents::COLLISION_EVENTS,
            ));
        }
        LayerFeature::Sprite(f) => {
            commands.entity(parent).insert((
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
