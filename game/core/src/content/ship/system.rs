use bevy::input::mouse::MouseWheel;
use bevy::math::FloatPow;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use std::collections::{HashMap, HashSet};
use std::f32::consts::PI;
use uuid::Uuid;

use crate::config::GameConfigValue;
use crate::constant::DROP_PICKUP_RADIUS;
#[cfg(debug_assertions)]
use crate::constant::LAYER_SHIP_UI;
use crate::content::block::feature::ammo::{Ammo, AmmoState};
use crate::content::block::feature::storage::{Storage, StorageState};
use crate::content::block::stat::StatsUpdateSet;
use crate::content::block::{
    Block, BlockDestroyedEvent, BlockOwner, BlockRegistry, DropComponent, DropEvent,
    ShipCalculatedStat,
};
use crate::content::death::Death;
use crate::content::ship::ai::{
    behavior_label, AiPlugin, BehaviorAttack, BehaviorChase, BehaviorFlee, BehaviorFollow,
    BehaviorGuard, BehaviorIdle, BehaviorKamikaze, BehaviorKiting, BehaviorPatrol, BehaviorRegroup,
    BehaviorWander,
};
use crate::content::ship::grid::ShipGrid;
use crate::content::ship::indicator::IndicatorsPlugin;
use crate::content::ship::stat::{ComputedShipStats, update_stats};
use crate::content::team::Team;
use crate::resource::{CameraTarget, ControlledShip, SceneState};
use crate::simulation::SimulationSet;

use super::*;

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ComputedShipStats>();
        #[cfg(debug_assertions)]
        app.add_systems(
            FixedUpdate,
            update_ship_ui
                .run_if(in_state(SceneState::Game))
                .in_set(SimulationSet::Input),
        );
        app.add_systems(
                PostUpdate,
                update_ship_behavior
                    .run_if(in_state(SceneState::Game))
                    .in_set(SimulationSet::Simulate),
            )
            .add_systems(
                FixedPreUpdate,
                update_stats
                    .after(StatsUpdateSet)
                    .run_if(in_state(SceneState::Game))
                    .in_set(SimulationSet::Simulate),
            )
            .add_systems(
                FixedUpdate,
                (
                    check_rebuild_ship_collider,
                    player_control_ship,
                )
                    .run_if(in_state(SceneState::Game))
                    .in_set(SimulationSet::Simulate),
            )
            .add_systems(
                FixedUpdate,
                collect_drop
                    .run_if(in_state(SceneState::Game))
                    .in_set(SimulationSet::Simulate),
            )
            .add_plugins((IndicatorsPlugin, AiPlugin));
    }
}

pub fn check_rebuild_ship_collider(
    mut commands: Commands,
    mut events: MessageReader<BlockDestroyedEvent>,
    mut ships: Query<(Entity, &mut Blocks, &mut ShipGrid, &GlobalTransform, &Team)>,
    ship_blocks: Query<(Entity, &Block, &BlockOwner)>,
    registry: Res<BlockRegistry>,
    mut drop_writer: MessageWriter<DropEvent>,
) {
    for event in events.read() {
        let Ok((entity, mut ship_block, mut grid, transform, team)) = ships.get_mut(event.ship)
        else {
            continue;
        };

        ship_block.blocks.retain(|f| &f.id != &event.block_id);
        grid.remove_block(event.block_id);

        let ShipCalculatedStat { ship, fragments } =
            registry.blocks_stats(&ship_block.blocks, &grid);

        let delete_blocks: HashSet<Uuid> = fragments
            .iter()
            .flat_map(|f| f.blocks.iter().map(|b| b.id))
            .collect();

        ship_block.blocks.retain(|f| !delete_blocks.contains(&f.id));

        if let Some(block) = registry.get_symbol(event.block_type.clone()) {
            if block.block_type.as_str() == "core" {
                commands.entity(entity).insert(Death);
            }
        }

        if let Some(ship) = ship {
            let collider = registry.build_ship_voxel_collider_2d(&grid);

            commands.entity(entity).remove::<Collider>().insert((
                ship.data.clone(),
                collider,
                ColliderMassProperties::Mass(ship.data.mass),
            ));
        }

        for (block_entity, block, owner) in ship_blocks.iter() {
            if delete_blocks.contains(&block.id) && owner.id == entity {
                commands.entity(block_entity).insert(Death);
            }
        }

        if !team.is_player() {
            for fragment in fragments {
                for block in &fragment.blocks {
                    let Some(def) = registry.get(&block.block_type) else {
                        continue;
                    };

                    let position = transform
                        .transform_point(block.position_pixel().extend(0.0))
                        .truncate();

                    for drop in &def.drops {
                        drop_writer.write(DropEvent {
                            position: position,
                            drop: drop.clone(),
                        });
                    }
                }
            }
        }
    }
}

#[cfg(debug_assertions)]
#[cfg(debug_assertions)]
pub fn update_ship_ui(
    mut commands: Commands,
    mut ui_roots: Query<(Entity, &ShipUIRoot, &mut Transform)>,
    ships: Query<(
        &Ship,
        Option<&BehaviorIdle>,
        Option<&BehaviorWander>,
        Option<&BehaviorKiting>,
        Option<&BehaviorRegroup>,
        Option<&BehaviorChase>,
        Option<&BehaviorAttack>,
        Option<&BehaviorPatrol>,
        Option<&BehaviorFlee>,
        Option<&BehaviorGuard>,
        Option<&BehaviorFollow>,
        Option<&BehaviorKamikaze>,
        Option<&ShipBehavior>,
        &GlobalTransform,
        &ComputedShipStats,
    )>,
    mut blocks: Query<(&BlockOwner, Option<&Ammo>, Option<&AmmoState>)>,
) {
    let mut data: HashMap<Entity, String> = HashMap::new();

    for (owner, ammo, ammo_state) in blocks.iter_mut() {
        let mut string = String::new();

        if let Some(ammo) = ammo {
            string.push_str(format!("\n{ammo:?}").as_str());
        }

        if let Some(ammo_state) = ammo_state {
            string.push_str(format!("\n{ammo_state:?}").as_str());
        }

        if !string.is_empty() {
            data.entry(owner.id)
                .and_modify(|f| f.push_str(string.as_str()))
                .or_insert(string);
        }
    }

    for (ui_entity, ui_root, _) in ui_roots.iter_mut() {
        if let Ok((
            ship,
            idle,
            wander,
            kiting,
            regroup,
            chase,
            attack,
            patrol,
            flee,
            guard,
            follow,
            kamikaze,
            behavior,
            global_transform,
            stats,
        )) = ships.get(ui_root.ship)
        {
            let current = behavior_label(
                idle, wander, kiting, regroup, chase, attack, patrol, flee, guard, follow,
                kamikaze,
            );
            let empty = String::new();
            let string = data.get(&ui_root.ship).unwrap_or(&empty);

            commands.entity(ui_entity).insert((
                Text2d::new(format!(
                    "Ship: {ship:?}\nC: {current}\nB: {behavior:?}\nS: {stats:?}\n{string}",
                )),
                TextFont {
                    font_size: FontSize::Px(30.0),
                    ..default()
                },
                Transform::from_rotation(global_transform.rotation().inverse())
                    .with_translation(Vec2::ZERO.extend(LAYER_SHIP_UI)),
            ));
        }
    }
}
pub fn player_control_ship(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut ships: Query<
        (&mut ExternalForce, &Transform, &Ship, &ComputedShipStats),
        Without<Camera2d>,
    >,
    mut camera: Query<(&mut Transform, &mut Projection), With<Camera2d>>,
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    mut camera_target: ResMut<CameraTarget>,
    mut controlled: ResMut<ControlledShip>,
    config: Res<GameConfigValue>,
) {
    for event in mouse_wheel_events.read() {
        if let Ok((_, mut projection)) = camera.single_mut() {
            if let Projection::Orthographic(ortho) = &mut *projection {
                let zoom_speed = 0.1;
                ortho.scale -= event.y * zoom_speed * ortho.scale;
                ortho.scale = ortho.scale.clamp(config.0.min_zoom, config.0.max_zoom);
            }
        }
    }

    if let Some(following) = camera_target.following {
        match ships.get(following) {
            Ok((_, transform, ship, _)) => {
                let target = transform
                    .transform_point(ship.core_position.extend(0.0))
                    .truncate();

                if let Ok((mut camera_transform, _)) = camera.single_mut() {
                    let current = camera_transform.translation.truncate();
                    let smooth_speed = 10.0;

                    let t = 1.0 - (-smooth_speed * time.delta_secs()).exp();
                    let new_position = current.lerp(target, t);

                    camera_transform.translation.x = new_position.x;
                    camera_transform.translation.y = new_position.y;
                }
            }
            Err(_) => {
                camera_target.following = None;
            }
        }
    } else {
        if let Ok((mut camera_transform, projection)) = camera.single_mut() {
            let scale = if let Projection::Orthographic(ortho) = &*projection {
                ortho.scale
            } else {
                1.0
            };

            let boost = if keyboard_input.pressed(KeyCode::ShiftLeft) {
                3.0
            } else {
                1.0
            };

            let pan_speed = 180.0 * scale * boost * time.delta_secs();

            if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
                camera_transform.translation.y += 1.0 * pan_speed;
            } else if keyboard_input.pressed(KeyCode::KeyS)
                || keyboard_input.pressed(KeyCode::ArrowDown)
            {
                camera_transform.translation.y -= 1.0 * pan_speed;
            }

            if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
                camera_transform.translation.x -= 1.0 * pan_speed;
            } else if keyboard_input.pressed(KeyCode::KeyD)
                || keyboard_input.pressed(KeyCode::ArrowRight)
            {
                camera_transform.translation.x += 1.0 * pan_speed;
            }
        }
    }

    if let Some(controlling) = controlled.controlling {
        match ships.get_mut(controlling) {
            Ok((mut force, transform, _, stats)) => {
                let thrust_power = stats.thrust_power;
                let turn_speed = stats.rotate_speed;

                let mut linear_force = Vec2::ZERO;
                let mut torque = 0.0;

                let forward = transform.local_y().truncate(); // Assuming +Y is forward
                let right = transform.local_x().truncate();

                if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp)
                {
                    linear_force += forward * thrust_power;
                }

                if keyboard_input.pressed(KeyCode::KeyS)
                    || keyboard_input.pressed(KeyCode::ArrowDown)
                {
                    linear_force -= forward * thrust_power;
                }

                if keyboard_input.pressed(KeyCode::KeyA)
                    || keyboard_input.pressed(KeyCode::ArrowLeft)
                {
                    torque += turn_speed;
                }

                if keyboard_input.pressed(KeyCode::KeyD)
                    || keyboard_input.pressed(KeyCode::ArrowRight)
                {
                    torque -= turn_speed;
                }

                // Strafe
                if keyboard_input.pressed(KeyCode::KeyQ) {
                    linear_force -= right * thrust_power;
                }
                if keyboard_input.pressed(KeyCode::KeyE) {
                    linear_force += right * thrust_power;
                }

                force.force = linear_force;
                force.torque = torque;
            }
            Err(_) => {
                controlled.controlling = None;
            }
        }
    } else {
    }
}

pub fn update_ship_behavior(
    mut ships: Query<(
        Entity,
        &Ship,
        &BoundingBox,
        &ShipBehavior,
        &Transform,
        &mut ExternalForce,
        &mut Velocity,
        &ComputedShipStats,
    )>,
    controlled: Res<ControlledShip>,
    #[cfg_attr(not(debug_assertions), allow(unused_variables))]
    mut gizmos: Gizmos,
    config: Res<GameConfigValue>,
) {
    for (entity, _, size, behavior, transform, mut force, mut velocity, stats) in ships.iter_mut() {
        if let Some(controlling) = controlled.controlling
            && controlling == entity
        {
            continue;
        }

        let thrust_power = stats.thrust_power;
        let turn_speed = stats.rotate_speed;

        let center = size.size / 2.0;

        let current_postion = transform.transform_point(center.extend(0.0)).truncate();
        let forward = transform.local_y().truncate();

        match behavior {
            ShipBehavior::Approach { position, radius } => {
                let direction = (position - current_postion).normalize_or_zero();
                let angle = direction.y.atan2(direction.x) - PI / 2.0;
                let angle_diff = wrap_angle(angle - transform.rotation.to_euler(EulerRot::XYZ).2);
                let distance = current_postion.distance(*position);

                if cfg!(debug_assertions) {
                    gizmos.line_2d(
                        current_postion,
                        *position,
                        Color::srgba(0.0, 1.0, 1.0, 0.5),
                    );

                    gizmos.circle_2d(*position, 30.0, Color::srgb(0.0, 1.0, 1.0));
                }

                let force_multiplier = if distance > *radius {
                    1.0
                } else {
                    (distance / radius).squared()
                };

                let torque_multiplier: f32 = (angle_diff.abs() / velocity.angular.abs())
                    .clamp(0.0, 1.0)
                    .squared();

                let mut torque: f32 = if angle_diff > 0.0 {
                    turn_speed
                } else {
                    -turn_speed
                } * torque_multiplier;

                if angle_diff.abs() < 0.01 && velocity.angular.abs() < 0.01 {
                    torque = 0.0;
                    velocity.angular = 0.0;
                }

                force.torque = torque;

                let speed = velocity.linear.length();
                let stop_distance =
                    speed * speed / (2.0 * config.0.linear_damping);

                if distance <= stop_distance {
                    force.force = Vec2::ZERO;
                } else if angle_diff.abs() < PI / 3.0 && distance > *radius {
                    force.force = forward * thrust_power * force_multiplier;
                } else {
                    force.force = Vec2::ZERO;
                }
            }
            ShipBehavior::GoAround { position, radius } => {
                let direction = (current_postion - position).normalize_or_zero();
                let position_on_ring = position + direction * radius;
                let mut next_position = position_on_ring;

                let in_range = current_postion.distance(position_on_ring) <= *radius;

                if in_range {
                    next_position = position_on_ring + direction.perp() * radius / 2.0;
                }

                if cfg!(debug_assertions) {
                    gizmos.line_2d(
                        current_postion,
                        next_position,
                        Color::srgba(0.0, 1.0, 0.0, 0.5),
                    );

                    gizmos.circle_2d(*position, *radius, Color::srgb(1.0, 1.0, 0.0));
                    gizmos.circle_2d(next_position, 30.0, Color::srgb(0.0, 1.0, 0.0));
                }

                let direction = (next_position - current_postion).normalize_or_zero();

                let angle = direction.y.atan2(direction.x) - PI / 2.0;
                let angle_diff = wrap_angle(angle - transform.rotation.to_euler(EulerRot::XYZ).2);
                let distance = current_postion.distance(next_position);

                let force_multiplier = if distance > *radius {
                    1.0
                } else {
                    distance / radius
                };

                let torque_multiplier: f32 = (angle_diff.abs() / velocity.angular.abs())
                    .clamp(0.0, 1.0)
                    .squared();

                let mut torque: f32 = if angle_diff > 0.0 {
                    turn_speed
                } else {
                    -turn_speed
                } * torque_multiplier;

                if angle_diff.abs() < 0.01 && velocity.angular.abs() < 0.01 {
                    torque = 0.0;
                    velocity.angular = 0.0;
                }

                force.torque = torque;

                if angle_diff.abs() < PI / 3.0 {
                    force.force = forward * thrust_power * force_multiplier;
                }
            }
        }
    }
}

fn wrap_angle(mut a: f32) -> f32 {
    while a > PI {
        a -= 2.0 * PI;
    }
    while a < -PI {
        a += 2.0 * PI;
    }
    a
}

pub fn collect_drop(
    mut commands: Commands,
    mut drops: Query<(Entity, &mut Transform, &DropComponent)>,
    mut storages: Query<(&Storage, &mut StorageState, &GlobalTransform, &Team)>,
    time: Res<Time>,
    config: Res<GameConfigValue>,
) {
    let mut collected: HashSet<Entity> = HashSet::new();

    for (storage, mut storage_state, ship_transform, ship_team) in storages.iter_mut() {
        if !ship_team.is_player() || storage_state.is_full(storage) {
            continue;
        }

        for (drop_entity, mut drop_transform, drop_component) in drops.iter_mut() {
            if collected.contains(&drop_entity) {
                continue;
            }

            let dir = ship_transform.translation() - drop_transform.translation;
            let distance = dir.length();

            if distance <= config.0.drop_collect_radius {
                drop_transform.translation +=
                    config.0.drop_collect_radius * dir.normalize_or_zero() * time.delta_secs() * 3.0;

                collected.insert(drop_entity);

                if distance <= DROP_PICKUP_RADIUS {
                    match drop_component {
                        DropComponent::Item(item) => {
                            if storage_state.add(storage, item.item_type.clone(), item.amount) {
                                commands.entity(drop_entity).despawn();
                            }
                        }
                    }
                }
            }
        }
    }
}
