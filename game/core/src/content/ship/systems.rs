use bevy::math::FloatPow;
use bevy::prelude::*;
use bevy::input::mouse::MouseWheel;
use bevy::time::common_conditions::on_timer;
use bevy_rapier2d::prelude::*;
use uuid::Uuid;
use std::time::Duration;
use std::collections::{HashMap, HashSet};
use std::f32::consts::PI;
use rand::Rng;
use rand::seq::IndexedRandom;

use crate::constant::{DROP_COLLECT_RADIUS, LAYER_SHIP_UI, LINEAR_DAMPING, MAX_SHIPS, MAX_ZOOM, MIN_ZOOM};
use crate::content::block::components::{
    Block, BlockDestroyedEvent, BlockOwner, BlockRegistry, BlockType, DropComponent,
    DropEvent, ShipCalculatedStat,
};
use crate::content::block::feature::ammo::{Ammo, AmmoState};
use crate::content::block::feature::storage::{Storage, StorageState};
use crate::content::block::stat::StatsUpdateSet;
use crate::content::death::Death;
use crate::content::ship::indicators::IndicatorsPlugin;
use crate::content::ship::stat::{ComputedShipStats, update_stats};
use crate::content::team::{DEFAULT_TEAM_ID, Team};
use crate::resource::{GameVars, SceneState};
use crate::save::io::load_saved_schematics;
use crate::utils::random::random_vec2_step;

use super::components::*;

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ComputedShipStats>()
            .add_systems(
                FixedUpdate,
                update_ship_ui.run_if(in_state(SceneState::Game)),
            )
            .add_systems(
                PostUpdate,
                update_ship_behavior.run_if(in_state(SceneState::Game)),
            )
            .add_systems(
                FixedPreUpdate,
                update_stats
                    .after(StatsUpdateSet)
                    .run_if(in_state(SceneState::Game)),
            )
            .add_systems(
                FixedUpdate,
                (
                    control_ship,
                    check_rebuild_ship_collider,
                    player_control_ship,
                )
                    .run_if(in_state(SceneState::Game)),
            )
            .add_systems(
                FixedUpdate,
                check_ship
                    .run_if(on_timer(Duration::from_millis(100)).and(in_state(SceneState::Game))),
            )
            .add_systems(FixedUpdate, collect_drop.run_if(in_state(SceneState::Game)))
            .add_plugins(IndicatorsPlugin);
    }
}

pub fn check_ship(
    mut commands: Commands,
    ships: Query<(&Ship, &Team)>,
    registry: Res<BlockRegistry>,
) {
    if ships.iter().len() >= MAX_SHIPS {
        return;
    }

    let mut rng = rand::rng();
    let schematics = load_saved_schematics();

    let mut maps: HashMap<u8, u8> = HashMap::new();

    for i in 0..2 {
        maps.insert(i, 0);
    }

    for (_, team) in ships {
        if let Some(value) = maps.get(&team.id) {
            maps.insert(team.id, value + 1);
        } else {
            maps.insert(team.id, 1);
        }
    }

    let mut least_team_id = DEFAULT_TEAM_ID;
    let mut least_count: u8 = MAX_SHIPS as u8;

    for (team_id, count) in maps.iter() {
        if *count < least_count {
            least_team_id = *team_id;
            least_count = *count;
        }
    }

    let Some(schematic) = schematics.choose(&mut rng) else {
        return;
    };

    let position = random_vec2_step(
        Vec2::new(-10000.0, -10000.0),
        Vec2::new(10000.0, 10000.0),
        1000.0,
    );

    schematic.to_ship_data().spawn(
        &mut commands,
        &registry,
        Team::new(least_team_id),
        Transform::from_translation(position.extend(0.0)),
    );
}

pub fn check_rebuild_ship_collider(
    mut commands: Commands,
    mut events: MessageReader<BlockDestroyedEvent>,
    mut ships: Query<(Entity, &mut Blocks, &GlobalTransform, &Team)>,
    ship_blocks: Query<(Entity, &Block, &BlockOwner)>,
    registry: Res<BlockRegistry>,
    mut drop_writer: MessageWriter<DropEvent>,
) {
    for event in events.read() {
        let Ok((entity, mut ship_block, transform, team)) = ships.get_mut(event.ship) else {
            continue;
        };

        ship_block.blocks.retain(|f| &f.id != &event.block_id);

        let ShipCalculatedStat { ship, fragments } = registry.blocks_stats(&ship_block.blocks);

        let delete_blocks: HashSet<Uuid> = fragments
            .iter()
            .flat_map(|f| f.blocks.iter().map(|b| b.id))
            .collect();

        ship_block.blocks.retain(|f| !delete_blocks.contains(&f.id));

        if let Some(block) = registry.get(&event.block_type) {
            if block.block_type == BlockType::Core {
                commands.entity(entity).insert(Death);
            }
        }

        if let Some(ship) = ship {
            let collider = registry.build_ship_voxel_collider_2d(&ship_block.blocks);

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

pub fn update_ship_ui(
    mut commands: Commands,
    mut ui_roots: Query<(Entity, &ShipUIRoot, &mut Transform)>,
    ships: Query<(
        Entity,
        &Ship,
        &AIShipController,
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
        match ships.get(ui_root.ship) {
            Ok((ship, _, controller, behavior, global_transform, stats)) => {
                let current = controller.current();

                let empty = String::new();
                let string = data.get(&ui_root.ship).unwrap_or(&empty);

                commands.entity(ui_entity).insert((
                    Text2d::new(format!(
                        "Ship: {ship}\nC: {:#?}\nB: {:#?}\nS: {:#?}\n{string}",
                        current, behavior, stats
                    )),
                    TextFont {
                        font_size: 30.0,
                        ..default()
                    },
                    Transform::from_rotation(global_transform.rotation().inverse())
                        .with_translation(Vec2::ZERO.extend(LAYER_SHIP_UI)),
                ));
            }
            Err(_) => {}
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
    mut state: ResMut<GameVars>,
) {
    for event in mouse_wheel_events.read() {
        if let Ok((_, mut projection)) = camera.single_mut() {
            if let Projection::Orthographic(ortho) = &mut *projection {
                let zoom_speed = 0.1;
                ortho.scale -= event.y * zoom_speed * ortho.scale;
                ortho.scale = ortho.scale.clamp(MIN_ZOOM, MAX_ZOOM);
            }
        }
    }

    if let Some(following) = state.following {
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
                state.following = None;
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

    if let Some(controlling) = state.controlling {
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
                state.controlling = None;
            }
        }
    } else {
    }
}

pub fn control_ship(
    time: Res<Time>,
    ships: Query<
        (
            Entity,
            &mut AIShipController,
            &mut ExternalForce,
            &Transform,
            &Ship,
            &BoundingBox,
            &Team,
        ),
        Without<Camera2d>,
    >,
    ships2: Query<(Entity, &GlobalTransform, &Ship, &BoundingBox, &Team)>,
    global_transforms: Query<&Transform, Without<Camera2d>>,
    state: ResMut<GameVars>,
    mut commands: Commands,
) {
    let mut teams: HashMap<u8, Vec<(Entity, &GlobalTransform, &Ship, &BoundingBox, &Team)>> =
        HashMap::new();
    let mut enemies: HashMap<u8, Vec<(Entity, &GlobalTransform, &Ship, &BoundingBox, &Team)>> =
        HashMap::new();

    let mut rng = rand::rng();

    for ship in ships2 {
        let value = teams.get_mut(&ship.4.id);

        if let Some(team) = value {
            team.push(ship);
        } else {
            teams.insert(ship.4.id, vec![ship]);
        }
    }

    for (team_id, _) in teams.iter() {
        let mut enemy = Vec::new();

        for (_, enemy_team) in teams.iter().filter(|f| f.0 != team_id) {
            enemy.extend(enemy_team.iter());
        }

        enemies.insert(*team_id, enemy);
    }

    for (entity, mut controller, _, transform, ship, bbox, team) in ships {
        if !controller.timer.tick(time.delta()).just_finished() {
            continue;
        }

        if let Some(following) = state.following
            && following == entity
        {
            continue;
        }

        if let Some(controlling) = state.controlling
            && controlling == entity
        {
            continue;
        }

        let ally = teams
            .get(&team.id)
            .expect("How this is possible")
            .iter()
            .filter(|other| other.0 != entity)
            .collect::<Vec<_>>();

        let mut in_range = enemies
            .get(&team.id)
            .expect("How this is possible")
            .iter()
            .filter(|other| {
                other
                    .1
                    .translation()
                    .truncate()
                    .distance(transform.translation.truncate())
                    <= ship.max_range
            });

        match controller.current() {
            AIShipBehavior::Regroup => {
                if let Some(a) = ally.last() {
                    let is_in_range = transform
                        .translation
                        .truncate()
                        .distance(a.1.translation().truncate())
                        <= (a.3.size.length() + bbox.size.length()) * 2.0;

                    if is_in_range {
                        controller.next(AIShipBehavior::Wander);
                    } else {
                        commands.entity(entity).insert(ShipBehavior::Approach {
                            position: a.1.transform_point((a.3.size / 2.0).extend(0.0)).truncate(),
                            radius: (a.3.size.length() + bbox.size.length()) * 2.0,
                        });
                    }
                } else {
                    controller.next(AIShipBehavior::Idle);
                }
            }
            AIShipBehavior::Idle => {
                if let Some(e) = in_range.next() {
                    controller.next(AIShipBehavior::Chase { target: e.0 });
                } else {
                    commands.entity(entity).insert(ShipBehavior::Approach {
                        position: Vec2::ZERO,
                        radius: bbox.size.length() * 2.0,
                    });
                }
            }
            AIShipBehavior::Chase { target } => {
                if target == entity {
                    panic!("Chase self");
                }

                let global_position = global_transforms
                    .get(target)
                    .map(|transform| transform.translation.truncate());

                if let Ok(position) = global_position {
                    if transform.translation.distance(position.extend(0.0)) >= ship.min_range {
                        commands.entity(entity).insert(ShipBehavior::Approach {
                            position: position,
                            radius: ship.min_range * 0.9,
                        });
                    } else {
                        commands.entity(entity).insert(ShipBehavior::GoAround {
                            position: position,
                            radius: ship.min_range * 0.9,
                        });
                    }
                } else {
                    controller.next(AIShipBehavior::Idle);
                }
            }
            AIShipBehavior::Wander => {
                if rng.random_bool(0.5) {
                    controller.next(AIShipBehavior::Attack {
                        target_position: Vec2::ZERO,
                    });
                } else if rng.random_bool(0.5) {
                    if let Some(a) = ally.last() {
                        controller.next(AIShipBehavior::Guard {
                            target: a.0,
                            radius: (a.3.size.length() + bbox.size.length()) * 2.0,
                        });
                    }
                } else {
                    controller.next(AIShipBehavior::Patrol {
                        around_position: Vec2::ZERO,
                        radius: bbox.size.length() * 10.0,
                    });
                }
            }
            AIShipBehavior::Attack { target_position } => {
                commands.entity(entity).insert(ShipBehavior::Approach {
                    position: target_position,
                    radius: bbox.size.length(),
                });
            }
            AIShipBehavior::Flee { from } => {
                let position = global_transforms
                    .get(from)
                    .map(|transform| transform.translation.truncate());

                if let Ok(position) = position {
                    let direction =
                        (transform.translation.truncate() - position).normalize_or_zero();

                    let op_direction = direction * -1.0;
                    let op_position = transform.translation.truncate() + op_direction * 1000.0;

                    commands.entity(entity).insert(ShipBehavior::Approach {
                        position: op_position,
                        radius: bbox.size.length(),
                    });
                }
            }
            AIShipBehavior::Patrol {
                around_position,
                radius,
            } => {
                commands.entity(entity).insert(ShipBehavior::Approach {
                    position: around_position,
                    radius,
                });
            }
            AIShipBehavior::Guard { target, radius } => {
                if let Ok(position) = global_transforms.get(target) {
                    let target_pos_actual = position.translation.truncate();

                    commands.entity(entity).insert(ShipBehavior::Approach {
                        position: target_pos_actual,
                        radius,
                    });
                } else {
                    controller.previous();
                }
            }
            AIShipBehavior::Follow { target, radius } => {
                if let Ok(position) = global_transforms.get(target) {
                    let target_pos_actual = position.translation.truncate();

                    commands.entity(entity).insert(ShipBehavior::Approach {
                        position: target_pos_actual,
                        radius,
                    });
                } else {
                    controller.previous();
                }
            }
            AIShipBehavior::Kamikaze { target } => {
                // Kamikaze at the target
                if let Ok(position) = global_transforms.get(target) {
                    let target_pos_actual = position.translation.truncate();

                    commands.entity(entity).insert(ShipBehavior::Approach {
                        position: target_pos_actual,
                        radius: bbox.size.length(),
                    });
                } else {
                    controller.previous();
                }
            }
        }
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
    state: Res<GameVars>,
    mut gizmos: Gizmos,
) {
    for (entity, _, size, behavior, transform, mut force, mut velocity, stats) in ships.iter_mut() {
        if let Some(controlling) = state.controlling
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

                gizmos.line_2d(
                    current_postion,
                    position.clone(),
                    Color::srgba(0.0, 1.0, 1.0, 0.5),
                );

                gizmos.circle_2d(position.clone(), 30.0, Color::srgb(0.0, 1.0, 1.0));

                let force_multiplier = if distance > *radius {
                    1.0
                } else {
                    (distance / radius).squared()
                };

                let torque_multiplier: f32 = (angle_diff.abs() / velocity.angvel.abs())
                    .clamp(0.0, 1.0)
                    .squared();

                let mut torque: f32 = if angle_diff > 0.0 {
                    turn_speed
                } else {
                    -turn_speed
                } * torque_multiplier;

                if angle_diff.abs() < 0.01 && velocity.angvel.abs() < 0.01 {
                    torque = 0.0;
                    velocity.angvel = 0.0;
                }

                force.torque = torque;

                let speed = velocity.linvel.length();
                let stop_distance = speed * speed / (2.0 * LINEAR_DAMPING);

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

                gizmos.line_2d(
                    current_postion,
                    next_position,
                    Color::srgba(0.0, 1.0, 0.0, 0.5),
                );

                gizmos.circle_2d(*position, *radius, Color::srgb(1.0, 1.0, 0.0));
                gizmos.circle_2d(next_position, 30.0, Color::srgb(0.0, 1.0, 0.0));

                let direction = (next_position - current_postion).normalize_or_zero();

                let angle = direction.y.atan2(direction.x) - PI / 2.0;
                let angle_diff = wrap_angle(angle - transform.rotation.to_euler(EulerRot::XYZ).2);
                let distance = current_postion.distance(next_position);

                let force_multiplier = if distance > *radius {
                    1.0
                } else {
                    distance / radius
                };

                let torque_multiplier: f32 = (angle_diff.abs() / velocity.angvel.abs())
                    .clamp(0.0, 1.0)
                    .squared();

                let mut torque: f32 = if angle_diff > 0.0 {
                    turn_speed
                } else {
                    -turn_speed
                } * torque_multiplier;

                if angle_diff.abs() < 0.01 && velocity.angvel.abs() < 0.01 {
                    torque = 0.0;
                    velocity.angvel = 0.0;
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

            if distance <= DROP_COLLECT_RADIUS {
                drop_transform.translation +=
                    DROP_COLLECT_RADIUS * dir.normalize_or_zero() * time.delta_secs() * 3.0;

                collected.insert(drop_entity);

                if distance <= DROP_COLLECT_RADIUS.sqrt() {
                    match drop_component {
                        DropComponent::Item(item) => {
                            if storage_state.add(storage, item.item_type, item.amount) {
                                commands.entity(drop_entity).despawn();
                            }
                        }
                    }
                }
            }
        }
    }
}
