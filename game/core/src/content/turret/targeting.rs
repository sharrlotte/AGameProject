use crate::content::{
    structures::ship::Enemy,
    team::Team,
    turret::{AIFireController, PlayerFireController, TargetingStrategy, TurretTargeter},
};
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

pub fn update_player_fire_controller(
    mut query: Query<(&mut TurretTargeter, &GlobalTransform), With<PlayerFireController>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    mouse_input: Res<ButtonInput<MouseButton>>,
) {
    let (camera, camera_transform) = *camera_query;

    if let Some(cursor_position) = window.cursor_position() {
        if let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) {
            for (mut targeter, _) in query.iter_mut() {
                targeter.target_pos = Some(world_position);
                targeter.active = mouse_input.pressed(MouseButton::Left);
            }
        }
    }
}

pub fn update_turret_target(
    mut turret_query: Query<(Entity, &GlobalTransform, &mut TurretTargeter, &Team)>,
    mut collision_events: MessageReader<CollisionEvent>,
    sensors: Query<Entity, With<TurretTargeter>>,
    enemies: Query<Entity, With<Enemy>>,
    global_transforms: Query<&GlobalTransform>,
) {
    for event in collision_events.read() {
        match event {
            CollisionEvent::Started(e1, e2, _flags) => {
                // Check if e1 or e2 is a DetectionRange sensor
                let sensor_enemy = match (
                    sensors.get(*e1).is_ok(),
                    enemies.get(*e1).is_ok(),
                    sensors.get(*e2).is_ok(),
                    enemies.get(*e2).is_ok(),
                ) {
                    (true, false, false, true) => Some((e1, e2)), // e1 = sensor, e2 = enemy
                    (false, true, true, false) => Some((e2, e1)), // e2 = sensor, e1 = enemy
                    _ => None,
                };

                if let Some((sensor, enemy)) = sensor_enemy {
                    // 👇 Now you know who is who
                    println!("Sensor {:?} detected enemy {:?}", sensor, enemy);
                    // Add enemy to potential targets
                    if let Ok((_, _, mut targeter, _)) = turret_query.get_mut(*sensor) {
                        targeter.potential_targets.insert(*enemy);
                    }
                }
            }
            CollisionEvent::Stopped(e1, e2, _flags) => {
                // Check if e1 or e2 is a DetectionRange sensor
                let sensor_enemy = match (
                    sensors.get(*e1).is_ok(),
                    enemies.get(*e1).is_ok(),
                    sensors.get(*e2).is_ok(),
                    enemies.get(*e2).is_ok(),
                ) {
                    (true, false, false, true) => Some((e1, e2)), // e1 = sensor, e2 = enemy
                    (false, true, true, false) => Some((e2, e1)), // e2 = sensor, e1 = enemy
                    _ => None,
                };

                if let Some((sensor, enemy)) = sensor_enemy {
                    // 👇 Now you know who is who
                    println!("Sensor {:?} end detected enemy {:?}", sensor, enemy);
                    // Add enemy to potential targets
                    if let Ok((_, _, mut targeter, _)) = turret_query.get_mut(*sensor) {
                        targeter.potential_targets.remove(enemy);
                    }
                }
            }
        }
    }

    // 2. Select best target from potential_targets
    for (_turret_entity, turret_transform, mut targeter, _turret_team) in turret_query.iter_mut() {
        let turret_pos = turret_transform.translation().truncate();
        let mut best_target: Option<Entity> = None;
        let mut closest_dist_sq = f32::MAX; // Use MAX to ensure we check all valid targets within logical range

        // Clean up invalid entities
        targeter
            .potential_targets
            .retain(|e| global_transforms.contains(*e));

        for &entity in targeter.potential_targets.iter() {
            if let Ok(target_transform) = global_transforms.get(entity) {
                let target_pos_actual = target_transform.translation().truncate();
                let dist_sq = turret_pos.distance_squared(target_pos_actual);

                // Optional: Double check range (physics might be slightly loose or delayed)
                if dist_sq > targeter.range * targeter.range {
                    continue;
                }

                match targeter.strategy {
                    TargetingStrategy::Closest => {
                        if dist_sq < closest_dist_sq {
                            closest_dist_sq = dist_sq;
                            best_target = Some(entity);
                        }
                    }
                    TargetingStrategy::MostHP => {
                        // Placeholder logic
                        if dist_sq < closest_dist_sq {
                            closest_dist_sq = dist_sq;
                            best_target = Some(entity);
                        }
                    }
                    TargetingStrategy::EnemyBullet => {
                        // Placeholder logic
                        if dist_sq < closest_dist_sq {
                            closest_dist_sq = dist_sq;
                            best_target = Some(entity);
                        }
                    }
                }
            }
        }

        targeter.target = best_target;
    }
}

pub fn apply_targeting_to_fire_targeter(
    mut query: Query<&mut TurretTargeter, With<AIFireController>>,
    target_query: Query<&GlobalTransform>,
) {
    for mut targeter in query.iter_mut() {
        if let Some(target_entity) = targeter.target {
            if let Ok(target_transform) = target_query.get(target_entity) {
                targeter.target_pos = Some(target_transform.translation().truncate());
                targeter.active = true;
            } else {
                targeter.active = false;
                targeter.target_pos = None;
            }
        } else {
            targeter.active = false;
            targeter.target_pos = None;
        }
    }
}
