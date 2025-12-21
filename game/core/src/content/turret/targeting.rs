use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    GameState, TurretControl,
    content::{
        block::base::{Block, TargetingStrategy},
        team::Team,
        turret::base::{Turret, TurretState},
    },
};

pub fn update_turret_target(
    camera_query: Single<(&Camera, &GlobalTransform)>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    game_state: Res<GameState>,

    mut collision_events: MessageReader<CollisionEvent>,
    mut turrets: Query<(Entity, &GlobalTransform, &Turret, &mut TurretState, &Team)>,
    mut blocks: Query<(Entity, &GlobalTransform, &Block)>,
) {
    for event in collision_events.read() {
        match event {
            CollisionEvent::Started(e1, e2, _flags) => {
                // Check if e1 or e2 is a DetectionRange sensor
                let (turret, block) = match (
                    turrets.contains(*e1),
                    blocks.contains(*e1),
                    turrets.contains(*e2),
                    blocks.contains(*e2),
                ) {
                    (true, _, _, true) => (turrets.get_mut(*e1), blocks.get_mut(*e2)), // e1 = sensor, e2 = enemy
                    (_, true, true, _) => (turrets.get_mut(*e2), blocks.get_mut(*e1)), // e2 = sensor, e1 = enemy
                    _ => continue,
                };

                // Add enemy to potential targets
                if let Ok((enemy, _, _)) = block
                    && let Ok((_, _, _, mut state, _)) = turret
                {
                    state.potential_targets.insert(enemy);
                }
            }
            CollisionEvent::Stopped(e1, e2, _flags) => {
                let (turret, block) = match (
                    turrets.contains(*e1),
                    blocks.contains(*e1),
                    turrets.contains(*e2),
                    blocks.contains(*e2),
                ) {
                    (true, _, _, true) => (turrets.get_mut(*e1), blocks.get_mut(*e2)), // e1 = sensor, e2 = enemy
                    (_, true, true, _) => (turrets.get_mut(*e2), blocks.get_mut(*e1)), // e2 = sensor, e1 = enemy
                    _ => continue,
                };

                if let Ok((enemy, _, _)) = block
                    && let Ok((_, _, _, mut state, _)) = turret
                {
                    state.potential_targets.remove(&enemy);
                }
            }
        }
    }

    // 2. Select best target from potential_targets
    for (_, transform, turret, mut state, team) in turrets.iter_mut() {
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

        let turret_position = transform.translation().truncate();

        let mut best_target: Option<Entity> = None;
        let mut closest_dist_sq = f32::MAX; // Use MAX to ensure we check all valid targets within logical range

        for &entity in state.potential_targets.iter() {
            if let Ok((_, target_transform, _)) = blocks.get(entity) {
                let target_pos_actual = target_transform.translation().truncate();
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

            // Atan2 returns angle in radians [-PI, PI] relative to X-axis
            // Assuming sprites face UP (Y+), we need to subtract PI/2
            let target_angle = diff.y.atan2(diff.x) - std::f32::consts::FRAC_PI_2;

            // Get current global rotation around Z
            let (_, current_rot, _) = global_transform.to_scale_rotation_translation();
            let current_angle = current_rot.to_euler(EulerRot::XYZ).2;

            let mut angle_error = target_angle - current_angle;

            // Normalize angle_error to [-PI, PI]
            while angle_error > std::f32::consts::PI {
                angle_error -= 2.0 * std::f32::consts::PI;
            }
            while angle_error < -std::f32::consts::PI {
                angle_error += 2.0 * std::f32::consts::PI;
            }

            let max_rot = turret.turn_speed * time.delta_secs();

            // Clamp rotation to max_rot, but don't overshoot
            let rot_amount = angle_error.clamp(-max_rot, max_rot);

            transform.rotate_z(rot_amount);
        }
    }
}
