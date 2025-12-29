use bevy::{math::FloatPow, prelude::*};
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

use crate::{
    content::{
        block::{
            base::{ShootPattern, TargetingStrategy},
            radar::Radar,
            stat::RotateSpeedComputed,
        },
        ship::base::Ship,
        team::Team,
    },
    resource::{GameVars, SceneState, TurretControl},
};

#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct Turret {
    pub range: f32,
    pub turn_speed: f32,
    pub barrels: Vec<Vec2>,
    pub strategy: TargetingStrategy,
    pub shooting_pattern: ShootPattern,
}

#[derive(Component, Debug, Clone, Reflect, Serialize, Deserialize, Default)]
pub struct TurretState {
    pub active: bool,
    pub target: Option<Entity>,
    pub shoot_at: Option<Vec2>,
}

impl TurretState {
    pub fn default() -> Self {
        Self {
            active: false,
            target: None,
            shoot_at: None,
        }
    }
}

pub struct TurretPlugin;

impl Plugin for TurretPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (turret_tracking, update_turret_target).run_if(in_state(SceneState::Game)),
        );
    }
}

pub fn update_turret_target(
    camera_query: Single<(&Camera, &GlobalTransform)>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    game_state: Res<GameVars>,
    ships: Query<(Entity, &GlobalTransform, &Ship)>,
    blocks: Query<(Entity, &Children)>,
    mut turrets: Query<(
        &GlobalTransform,
        &Turret,
        &mut TurretState,
        &Team,
        &Radar,
        Option<&RotateSpeedComputed>,
    )>,
) {
    for (transform, turret, mut state, team, radar, rotate_speed) in turrets.iter_mut() {
        if team.is_player() {
            let (camera, camera_transform) = *camera_query;

            if let Some(cursor_position) = window.cursor_position()
                && let Ok(world_position) =
                    camera.viewport_to_world_2d(camera_transform, cursor_position)
                && world_position.distance_squared(transform.translation().truncate())
                    <= turret.range.squared()
            {
                match game_state.turret_control {
                    TurretControl::Manual => {
                        state.shoot_at = Some(world_position);
                        state.active = mouse_input.pressed(MouseButton::Left);
                        continue;
                    }
                    TurretControl::Hold => {
                        if mouse_input.pressed(MouseButton::Left) {
                            state.shoot_at = Some(world_position);
                            state.active = true;
                            continue;
                        }
                    }
                    TurretControl::Auto => {}
                }
            }
        }

        let mut best_target: Option<Entity> = None;

        let mut closest_dist_sq = turret.range.squared();

        for &entity in radar.targets.iter() {
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

        if let Some(entity) = best_target
            && let Ok((_, target_transform, _)) = ships.get(entity)
        {
            let to_target =
                target_transform.translation().truncate() - transform.translation().truncate();
            let target_dir = to_target.normalize_or_zero();
            let forward: Vec2 = transform.up().truncate();
            let angle_error: f32 = forward.perp_dot(target_dir).atan2(forward.dot(target_dir));

            state.target = Some(entity);

            let turn_speed = rotate_speed.map(|r| r.value).unwrap_or(turret.turn_speed);

            if turn_speed == 0. {
                state.active = angle_error.abs() < PI / 64.0;
            } else {
                state.active = angle_error.abs() < PI / 32.0;
            }
        } else {
            state.target = None;
            state.active = false;
        }
    }
}

pub fn turret_tracking(
    time: Res<Time>,
    mut query: Query<(
        &mut Transform,
        &GlobalTransform,
        &Turret,
        &TurretState,
        Option<&RotateSpeedComputed>,
    )>,
    transforms: Query<(&Ship, &GlobalTransform)>,
) {
    for (mut transform, global_transform, turret, state, rotate_speed) in &mut query {
        let turn_speed = rotate_speed.map(|r| r.value).unwrap_or(turret.turn_speed);

        if turn_speed == 0. {
            continue;
        }

        let mut target_position: Option<Vec2> = None;

        if let Some(shoot_at) = state.shoot_at {
            target_position = Some(shoot_at);
        } else if let Some(target) = state.target {
            if let Ok((ship, position)) = transforms.get(target) {
                target_position = Some(
                    position
                        .transform_point(ship.core_position.extend(0.0))
                        .truncate(),
                );
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

            let target_dir = to_target.normalize_or_zero();
            let angle_error: f32 = forward.perp_dot(target_dir).atan2(forward.dot(target_dir));
            let max_turn = turn_speed * time.delta_secs();
            let turn = angle_error.clamp(-max_turn, max_turn);

            transform.rotate_z(turn);
        }
    }
}
