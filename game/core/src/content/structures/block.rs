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

pub fn render_layer_feature(
    feature: &LayerFeature,
    commands: &mut Commands,
    entity: Entity,
    team: Team,
) {
    if team.id == Team::default().id {
        commands.entity(entity).insert(PlayerFireController);
    } else {
        commands.entity(entity).insert(AIFireController);
    }

    commands.entity(entity).insert(team);

    match feature {
        LayerFeature::Recoil(_recoil_feature) => todo!(),
        LayerFeature::HeatEmitter(_heat_emitter_feature) => todo!(),
        LayerFeature::EnergyConsumer(_energy_consumer_feature) => todo!(),
        LayerFeature::Turret(f) => {
            commands.entity(entity).insert((
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
                    target_pos: None,
                },
                Collider::ball(f.range),
                CollisionGroups::new(team.get_radar_group(), team.get_radar_filter()),
                Sensor,
                ColliderMassProperties::Mass(0.0),
                ActiveEvents::COLLISION_EVENTS,
            ));
        }
    }
}

pub fn render_block_definition(
    def: &BlockDefinition,
    commands: &mut Commands,
    asset_server: &AssetServer,
    grid_pos: IVec2,
    team: Team,
) -> Entity {
    let size_vec = def.shape.size.as_vec2();
    let half_size = size_vec * GRID_SIZE * 0.5;

    let anchor_pos = Vec3::new(
        grid_pos.x as f32 * GRID_SIZE,
        grid_pos.y as f32 * GRID_SIZE,
        0.0,
    );

    let pos = anchor_pos + half_size.extend(0.0);

    let entity = commands
        .spawn((
            Transform::from_translation(pos),
            Visibility::Visible,
            GridPosition(grid_pos),
            PlacedBlock,
            ShipBlockHitbox,
            Collider::cuboid(half_size.x, half_size.y),
            CollisionGroups::new(team.get_block_group(), team.get_block_filter()),
            ColliderMassProperties::Mass(0.0),
            Block {
                rotation: 0,
                block_type: def.block_type,
            },
            Health {
                health: def.health,
                max_health: def.health,
            },
            def.block_group,
            team,
        ))
        .id();

    for layer in &def.layers {
        let layer_entity = commands
            .spawn((
                BlockLayer {
                    _kind: layer.kind.clone(),
                },
                Transform::default(),
                GlobalTransform::default(),
                Visibility::Inherited,
            ))
            .id();

        commands.entity(entity).add_child(layer_entity);

        // Visuals
        for visual in &layer.visuals {
            let visual_entity = commands
                .spawn((
                    Sprite {
                        image: asset_server.load(&visual.sprite),
                        custom_size: Some(size_vec * GRID_SIZE),
                        color: Color::srgb(1.005, 1.005, 1.005),
                        ..default()
                    },
                    Transform::from_translation(visual.offset.extend(visual.z)),
                ))
                .id();

            commands.entity(layer_entity).add_child(visual_entity);
        }

        // Features
        for feature in &layer.features {
            render_layer_feature(feature, commands, layer_entity, team);
        }
    }

    return entity;
}
