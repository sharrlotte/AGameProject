use crate::common::death::component::Death;
use crate::constant::{THRUST_POWER_FACTOR, TURN_SPEED_FACTOR};
use crate::content::block::component::BlockType;
use crate::content::block::message::BlockDestroyedEvent;
use crate::content::block::resource::{BlockRegistry, ShipCalculatedStat};
use crate::content::ship::component::{AIShipController, PlayerShipController, Ship};
use crate::content::team::Team;
use crate::resource::GameState;
use crate::save::io::load_saved_schematics;
use crate::utils::random::random_vec2_step;
use bevy::prelude::*;
use bevy_rapier2d::prelude::{Collider, ColliderMassProperties};
use rand::Rng;
use rand::seq::IndexedRandom;
use std::f32::consts::PI;

use bevy::math::FloatPow;
use bevy_rapier2d::prelude::*;

use crate::content::ship::component::AIShipBehavior;

pub fn check_player_ship(
    mut commands: Commands,
    ships: Query<&PlayerShipController>,
    vars: Res<GameState>,
    registry: Res<BlockRegistry>,
) {
    if ships.iter().len() == 0 {
        let ship = if let Some(ship) = &vars.current_ship {
            ship
        } else {
            warn!("No ship loaded in GameState!");
            return;
        };

        let position = random_vec2_step(
            Vec2::new(-10000.0, -10000.0),
            Vec2::new(10000.0, 10000.0),
            5000.0,
        );

        ship.to_ship_data().spawn(
            &mut commands,
            &registry,
            Team::new(0),
            Transform::from_translation(position.extend(0.0)),
        );
    }
}

const ENEMIES: usize = 20;

pub fn check_enemy_ship(
    mut commands: Commands,
    ships: Query<&AIShipController>,
    registry: Res<BlockRegistry>,
) {
    if ships.iter().len() > ENEMIES {
        return;
    }

    let missing = ENEMIES - ships.iter().len();
    let mut rng = rand::rng();

    for _ in 0..missing {
        let team_id = rng.random_range(1..10);
        let schematics = load_saved_schematics();
        let Some(schematic) = schematics.choose(&mut rng) else {
            continue;
        };

        let position = random_vec2_step(
            Vec2::new(-10000.0, -10000.0),
            Vec2::new(10000.0, 10000.0),
            500.0,
        );

        schematic.to_ship_data().spawn(
            &mut commands,
            &registry,
            Team::new(team_id),
            Transform::from_translation(position.extend(0.0)),
        );
    }
}

pub fn cleanup_ships(mut commands: Commands, ships: Query<Entity, With<Ship>>) {
    for entity in &ships {
        commands.entity(entity).insert(Death);
    }
}

pub fn check_rebuild_ship_collider(
    mut commands: Commands,
    mut events: MessageReader<BlockDestroyedEvent>,
    mut ships: Query<(Entity, &mut Ship, &Team)>,
    registry: Res<BlockRegistry>,
) {
    for event in events.read() {
        let Ok((entity, mut ship, _)) = ships.get_mut(event.ship_id) else {
            continue;
        };

        if let Some(block) = registry.get(&event.block) {
            if block.block_type == BlockType::Core {
                commands.entity(entity).insert(Death);
                continue;
            }
        }

        ship.blocks.retain_mut(|f| f.id != event.block_id);

        let ShipCalculatedStat {
            mass,
            center,
            collider,
            thrust_power,
            turn_speed,
        } = registry.blocks_stats(&ship.blocks);

        commands.entity(entity).remove::<Collider>().insert((
            collider,
            ColliderMassProperties::Mass(mass),
            Ship {
                id: ship.id,
                blocks: ship.blocks.clone(),
                center,
                thrust_power,
                turn_speed,
                radar_range: 3000.0,
            },
        ));
    }
}

pub fn player_ship_controller(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut ExternalForce, &Transform, &Ship), With<PlayerShipController>>,
) {
    for (mut force, transform, ship) in query.iter_mut() {
        let thrust_power = ship.thrust_power * THRUST_POWER_FACTOR;
        let turn_speed = ship.turn_speed * TURN_SPEED_FACTOR;

        let mut linear_force = Vec2::ZERO;
        let mut torque = 0.0;

        let forward = transform.local_y().truncate(); // Assuming +Y is forward
        let right = transform.local_x().truncate();

        if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
            linear_force += forward * thrust_power;
        }

        if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
            linear_force -= forward * thrust_power;
        }

        if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
            torque += turn_speed;
        }

        if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
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
}

pub fn ai_ship_controller(
    mut query: Query<(
        &mut AIShipController,
        &mut ExternalForce,
        &Velocity,
        &Transform,
        &Ship,
        &Team,
    )>,
    ships: Query<(Entity, &Transform, &Ship, &Team)>,
    global_transforms: Query<&Transform>,
) {
    for (mut controller, mut force, velocity, transform, ship, team) in query.iter_mut() {
        let thrust_power = ship.thrust_power * THRUST_POWER_FACTOR;
        let turn_speed = ship.turn_speed * TURN_SPEED_FACTOR;

        let enemy_ships: Vec<(Entity, &Transform, &Ship, &Team)> = ships
            .iter()
            .filter(|(_, _, _, other_team)| other_team.id != team.id)
            .collect();

        let in_range: Vec<(Entity, &Transform, &Ship, &Team)> = enemy_ships
            .into_iter()
            .filter(|(_, enemy_transform, _, _)| {
                enemy_transform
                    .translation
                    .truncate()
                    .distance(transform.translation.truncate())
                    <= ship.radar_range
            })
            .collect();

        match controller.current() {
            AIShipBehavior::Idle => {
                if !in_range.is_empty() {
                    controller.next(AIShipBehavior::Chase {
                        target: in_range[0].0,
                    });
                }
            }
            AIShipBehavior::Chase { target } => {
                let position = global_transforms
                    .get(target)
                    .map(|transform| transform.translation.truncate());

                if let Ok(position) = position {
                    approach(
                        transform,
                        velocity,
                        &mut force,
                        position,
                        thrust_power,
                        turn_speed,
                        100.0,
                    );
                } else {
                    controller.reset();
                }
            }
            AIShipBehavior::Wander => {}
            AIShipBehavior::Attack { target_position } => {
                approach(
                    transform,
                    velocity,
                    &mut force,
                    target_position,
                    thrust_power,
                    turn_speed,
                    100.0,
                );
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

                    approach(
                        transform,
                        velocity,
                        &mut force,
                        op_position,
                        thrust_power,
                        turn_speed,
                        100.0,
                    );
                }
            }
            AIShipBehavior::Patrol {
                around_position,
                radius,
            } => {
                go_around(
                    transform,
                    velocity,
                    &mut force,
                    around_position,
                    radius,
                    thrust_power,
                    turn_speed,
                );
            }
            AIShipBehavior::Guard { target, radius } => {
                if let Ok(position) = global_transforms.get(target) {
                    let target_pos_actual = position.translation.truncate();

                    go_around(
                        transform,
                        velocity,
                        &mut force,
                        target_pos_actual,
                        radius,
                        thrust_power,
                        turn_speed,
                    );
                } else {
                    controller.reset();
                }
            }
            AIShipBehavior::Follow { target, radius } => {
                if let Ok(position) = global_transforms.get(target) {
                    let target_pos_actual = position.translation.truncate();

                    approach(
                        transform,
                        velocity,
                        &mut force,
                        target_pos_actual,
                        thrust_power,
                        turn_speed,
                        radius,
                    );
                } else {
                    controller.reset();
                }
            }
            AIShipBehavior::Kamikaze { target } => {
                // Kamikaze at the target
                if let Ok(position) = global_transforms.get(target) {
                    let target_pos_actual = position.translation.truncate();

                    approach(
                        transform,
                        velocity,
                        &mut force,
                        target_pos_actual,
                        thrust_power,
                        turn_speed,
                        0.0,
                    );
                } else {
                    controller.reset();
                }
            }
        }
    }
}

fn approach(
    transform: &Transform,
    velocity: &Velocity,
    force: &mut ExternalForce,
    target_position: Vec2,
    thrust_power: f32,
    turn_speed: f32,
    radius: f32,
) {
    let direction = (target_position - transform.translation.truncate()).normalize_or_zero();
    let angle = direction.y.atan2(direction.x) - PI / 2.0;
    let angle_diff = angle - transform.rotation.to_euler(EulerRot::XYZ).2;
    let distance = transform.translation.truncate().distance(target_position);

    let turn_speed_multiplier = (angle_diff.abs() / PI).clamp(0.25, 1.0);
    let force_multiplier = if distance > radius.squared().squared() {
        1.0
    } else if distance > radius.squared() {
        0.5
    } else {
        0.1
    };

    let torque = (if angle_diff > 0.0 {
        turn_speed
    } else {
        -turn_speed
    }) * (if velocity.linvel.length() > PI / 4.0 {
        0.1
    } else {
        1.0
    }) * turn_speed_multiplier;

    force.torque = torque;

    if angle_diff.abs() < PI / 2.0 {
        if distance <= radius {
            force.force = Vec2::ZERO;
        } else {
            force.force = direction * thrust_power * force_multiplier;
        }
    }
}

fn go_around(
    transform: &Transform,
    velocity: &Velocity,
    force: &mut ExternalForce,
    around_position: Vec2,
    radius: f32,
    thrust_power: f32,
    turn_speed: f32,
) {
    let in_range = transform.translation.truncate().distance(around_position) <= 300.0;

    let direction = (around_position - transform.translation.truncate()).normalize_or_zero();

    let position_on_ring = transform.translation.truncate() + direction * radius;

    if in_range {
        let next_position = position_on_ring + Vec2::new(direction.y, -direction.x) * 100.0;

        approach(
            transform,
            velocity,
            force,
            next_position,
            thrust_power,
            turn_speed,
            100.0,
        );
    } else {
        approach(
            transform,
            velocity,
            force,
            position_on_ring,
            thrust_power,
            turn_speed,
            100.0,
        );
    }
}
