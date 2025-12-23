use bevy::{math::FloatPow, prelude::*};
use bevy_rapier2d::prelude::*;

use crate::{
    GameState, TurretControl,
    content::{
        block::base::TargetingStrategy,
        ship::base::Ship,
        team::Team,
        turret::base::{PotentialTargetChange, Turret, TurretState},
    },
};

pub fn update_turret_target(
    mut collision_events: MessageReader<CollisionEvent>,
    mut turrets: Query<(Entity, &mut TurretState), With<Turret>>,
    mut ships: Query<Entity, With<Ship>>,
    mut potential_target_events: MessageWriter<PotentialTargetChange>,
) {

    for event in collision_events.read() {
        match event {
            CollisionEvent::Started(e1, e2, _flags) => {
                let (turret, block) = match (
                    turrets.contains(*e1),
                    ships.contains(*e1),
                    turrets.contains(*e2),
                    ships.contains(*e2),
                ) {
                    (true, _, _, true) => (turrets.get_mut(*e1), ships.get_mut(*e2)), // e1 = sensor, e2 = enemy
                    (_, true, true, _) => (turrets.get_mut(*e2), ships.get_mut(*e1)), // e2 = sensor, e1 = enemy
                    _ => continue,
                };

                // Add enemy to potential targets
                if let Ok(enemy) = block
                    && let Ok((turret, mut state)) = turret
                {
                    state.potential_targets.insert(enemy);
                    potential_target_events.write(PotentialTargetChange { turret });
                }
            }
            CollisionEvent::Stopped(e1, e2, _flags) => {
                let (turret, block) = match (
                    turrets.contains(*e1),
                    ships.contains(*e1),
                    turrets.contains(*e2),
                    ships.contains(*e2),
                ) {
                    (true, _, _, true) => (turrets.get_mut(*e1), ships.get_mut(*e2)), // e1 = sensor, e2 = enemy
                    (_, true, true, _) => (turrets.get_mut(*e2), ships.get_mut(*e1)), // e2 = sensor, e1 = enemy
                    _ => continue,
                };

                if let Ok(enemy) = block
                    && let Ok((turret, mut state)) = turret
                {
                    state.potential_targets.remove(&enemy);
                    potential_target_events.write(PotentialTargetChange { turret });
                }
            }
        }
    }
}

pub fn update_best_target(
    camera_query: Single<(&Camera, &GlobalTransform)>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    game_state: Res<GameState>,
    ships: Query<(Entity, &GlobalTransform, &Ship)>,
    mut turrets: Query<(Entity, &GlobalTransform, &Turret, &mut TurretState, &Team)>,
    mut potential_target_events: MessageReader<PotentialTargetChange>,
) {
    for (_, _, _, mut state, team) in turrets.iter_mut() {
        if team.is_player() {
            let (camera, camera_transform) = *camera_query;

            if let Some(cursor_position) = window.cursor_position() {
                if let Ok(world_position) =
                    camera.viewport_to_world_2d(camera_transform, cursor_position)
                {
                    match game_state.turret_control {
                        TurretControl::Auto => {}
                        TurretControl::Manual => {
                            state.shoot_at = Some(world_position);
                            state.active = mouse_input.pressed(MouseButton::Left);
                            continue;
                        }
                        TurretControl::Hold => {
                            if mouse_input.pressed(MouseButton::Left) {
                                state.shoot_at = Some(world_position);
                                state.active = true;
                            }
                        }
                    }
                }
            }
        }
    }

    for event in potential_target_events.read() {
        if let Ok((_, transform, turret, mut state, _)) = turrets.get_mut(event.turret) {
            // state.potential_targets.remove(&event.turret);

            let mut best_target: Option<Entity> = None;

            let mut closest_dist_sq = turret.range.squared();

            for &entity in state.potential_targets.iter() {
                //TODO: FIX perf
                if let Ok((_, target_transform, _)) = ships.get(entity) {
                    let target_pos_actual = target_transform.translation().truncate();
                    let turret_position = transform.translation().truncate();
                    let dist_sq = turret_position.distance_squared(target_pos_actual);

                    if dist_sq > turret.range * turret.range {
                        continue;
                    }

                    match turret.strategy {
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

            if let Some(entity) = best_target {
                state.target = Some(entity);
                state.active = true;
            } else {
                state.target = None;
                state.active = false;
            }
        }
    }
}

pub fn system_turret_tracking(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &GlobalTransform, &Turret, &TurretState)>,
    transforms: Query<&GlobalTransform>,
) {
    for (mut transform, global_transform, turret, state) in &mut query {
        let mut target_position: Option<Vec2> = None;
        if let Some(shoot_at) = state.shoot_at {
            target_position = Some(shoot_at);
        } else if let Some(target) = state.target {
            if let Ok(position) = transforms.get(target) {
                target_position = Some(position.translation().truncate());
            }
        }

        if let Some(shoot_at_position) = target_position {
            let current_position = global_transform.translation().truncate();
            let diff = shoot_at_position - current_position;
            let dist_sq = diff.length_squared();

            if dist_sq < 0.0001 {
                continue;
            }

            let forward: Vec2 = global_transform.up().truncate();

            let to_target = shoot_at_position - global_transform.translation().truncate();

            if to_target.length_squared() < 0.0001 {
                continue;
            }

            let target_dir = to_target.normalize();
            let angle_error = forward.perp_dot(target_dir).atan2(forward.dot(target_dir));
            let max_turn = turret.turn_speed * time.delta_secs();
            let turn = angle_error.clamp(-max_turn, max_turn);

            transform.rotate_z(turn);
        }
    }
}
