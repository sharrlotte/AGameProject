use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;
use data::GameState;
use world::{AimDirection, Health};

use crate::anim::PlayerVisualRoot;
use crate::anim::blend::approach;
use crate::anim::catalog::{AnimCatalog, TwistConfig};
use crate::config::VisualsConfig;

/// Locomotion orientation zone derived from the angle between movement and aim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MoveZone {
    #[default]
    Forward,
    Sideways,
    Backward,
}

/// Independent lower-body and upper-body yaw state kept on `PlayerVisualRoot`.
#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerFacing {
    pub lower: Quat,
    pub upper: Quat,
    pub twist_rad: f32,
    pub zone: MoveZone,
    pub idle_turning: bool,
}

impl Default for PlayerFacing {
    fn default() -> Self {
        Self {
            lower: Quat::IDENTITY,
            upper: Quat::IDENTITY,
            twist_rad: 0.0,
            zone: MoveZone::Forward,
            idle_turning: false,
        }
    }
}

/// Adds facing state to every player visual root, including roots spawned without animation assets.
pub fn init_player_facing(
    mut commands: Commands,
    roots: Query<(Entity, Option<&PlayerFacing>), With<PlayerVisualRoot>>,
) {
    for (entity, facing) in &roots {
        if facing.is_none() {
            commands.entity(entity).insert(PlayerFacing::default());
        }
    }
}

/// Updates lower-body facing, upper-body aim facing, and residual spine twist.
pub fn sync_player_facing(
    time: Res<Time>,
    state: Option<Res<State<GameState>>>,
    catalog: Res<AnimCatalog>,
    cfg: Res<VisualsConfig>,
    survivors: Query<(&AimDirection, &Velocity, &Health)>,
    mut roots: Query<(&ChildOf, &mut Transform, &mut PlayerFacing), With<PlayerVisualRoot>>,
) {
    let dt = time.delta_secs();
    let game_over = state
        .as_deref()
        .map(|s| *s.get() == GameState::GameOver)
        .unwrap_or(false);
    let threshold_sq = cfg.player.move_speed_threshold.max(0.0).powi(2);

    for (child_of, mut transform, mut facing) in &mut roots {
        let Ok((aim, velocity, health)) = survivors.get(child_of.parent()) else {
            continue;
        };

        let alive = health.current > 0.0 && !game_over;
        let horizontal = Vec3::new(velocity.linear.x, 0.0, velocity.linear.z);
        let move_dir = (horizontal.length_squared() > threshold_sq).then(|| horizontal.normalize());
        let aim_dir = (aim.0.length_squared() > 1e-4).then(|| aim.0.normalize());

        step_facing(
            &mut facing,
            aim_dir,
            move_dir,
            alive,
            dt,
            catalog.facing_blend_secs,
            &catalog.twist,
        );
        transform.rotation = facing.lower;
    }
}

/// Pure facing step updating upper yaw, cone-based lower yaw, idle turn catch-up, and twist.
pub fn step_facing(
    facing: &mut PlayerFacing,
    aim_dir: Option<Vec3>,
    move_dir: Option<Vec3>,
    alive: bool,
    dt: f32,
    facing_blend_secs: f32,
    twist: &TwistConfig,
) {
    if !alive {
        facing.twist_rad = 0.0;
        facing.idle_turning = false;
        return;
    }

    if let Some(dir) = aim_dir
        && let Some(upper) = horizontal_yaw(dir)
    {
        facing.upper = upper;
    }

    let max_twist_deg = twist.max_twist_deg.clamp(1.0, 90.0);
    let max_twist_rad = max_twist_deg.to_radians();

    let target_lower = if let Some(move_vec) = move_dir {
        facing.idle_turning = false;
        let move_yaw = horizontal_yaw(move_vec).unwrap_or(facing.lower);
        let upper_dir = facing.upper * Vec3::NEG_Z;
        let abs_deg = signed_angle_xz(move_vec, upper_dir).abs().to_degrees();
        facing.zone = next_move_zone(facing.zone, abs_deg, max_twist_deg, twist.hysteresis_deg);
        match facing.zone {
            MoveZone::Forward => move_yaw,
            MoveZone::Sideways => facing.upper,
            MoveZone::Backward => horizontal_yaw(-move_vec).unwrap_or(facing.upper),
        }
    } else {
        let current_delta_deg = signed_yaw_delta(facing.lower, facing.upper)
            .abs()
            .to_degrees();
        if current_delta_deg > max_twist_deg {
            facing.idle_turning = true;
        }
        if facing.idle_turning {
            facing.upper
        } else {
            facing.lower
        }
    };

    let lower_alpha = slerp_alpha(dt, facing_blend_secs);
    facing.lower = facing.lower.slerp(target_lower, lower_alpha).normalize();

    if move_dir.is_none() && facing.idle_turning {
        let remaining_deg = signed_yaw_delta(facing.lower, facing.upper)
            .abs()
            .to_degrees();
        if remaining_deg <= twist.idle_turn_stop_deg.max(0.0) {
            facing.idle_turning = false;
        }
    }

    let target_twist = if twist.enabled {
        signed_yaw_delta(facing.lower, facing.upper).clamp(-max_twist_rad, max_twist_rad)
    } else {
        0.0
    };
    facing.twist_rad = approach(facing.twist_rad, target_twist, dt, twist.twist_blend_secs);
}

fn next_move_zone(current: MoveZone, abs_deg: f32, max_deg: f32, hyst_deg: f32) -> MoveZone {
    let hyst = hyst_deg.max(0.0);
    let back_start = (180.0 - max_deg).max(max_deg);

    match current {
        MoveZone::Forward => {
            if abs_deg <= max_deg + hyst {
                MoveZone::Forward
            } else if abs_deg > back_start + hyst {
                MoveZone::Backward
            } else {
                MoveZone::Sideways
            }
        }
        MoveZone::Sideways => {
            if abs_deg < max_deg - hyst {
                MoveZone::Forward
            } else if abs_deg > back_start + hyst {
                MoveZone::Backward
            } else {
                MoveZone::Sideways
            }
        }
        MoveZone::Backward => {
            if abs_deg >= back_start - hyst {
                MoveZone::Backward
            } else if abs_deg < max_deg - hyst {
                MoveZone::Forward
            } else {
                MoveZone::Sideways
            }
        }
    }
}

fn slerp_alpha(dt: f32, blend_secs: f32) -> f32 {
    if blend_secs <= 0.0 || dt <= 0.0 {
        1.0
    } else {
        (1.0 - (-dt / blend_secs).exp()).clamp(0.0, 1.0)
    }
}

fn horizontal_yaw(dir: Vec3) -> Option<Quat> {
    let flat = Vec3::new(dir.x, 0.0, dir.z);
    if flat.length_squared() <= 1e-4 {
        return None;
    }
    let norm = flat.normalize();
    Some(yaw_from_neg_z(norm))
}

/// Computes a pure Y-axis rotation from `-Z` toward a normalized horizontal `dir`.
fn yaw_from_neg_z(dir: Vec3) -> Quat {
    Quat::from_rotation_y((-dir.x).atan2(-dir.z))
}

/// Signed horizontal Y-rotation from `from` to `to` in `[-π, π]`.
fn signed_yaw_delta(from: Quat, to: Quat) -> f32 {
    signed_angle_xz(from * Vec3::NEG_Z, to * Vec3::NEG_Z)
}

/// Signed Y-axis angle in `[-π, π]` rotating horizontal `from` onto `to`.
fn signed_angle_xz(from: Vec3, to: Vec3) -> f32 {
    let a = Vec2::new(from.x, from.z).normalize_or_zero();
    let b = Vec2::new(to.x, to.z).normalize_or_zero();
    if a == Vec2::ZERO || b == Vec2::ZERO {
        return 0.0;
    }
    // Right-handed Y-up: rotating (0, -1) by +theta around +Y gives (-sin, -cos),
    // so cross_y = a.z * b.x - a.x * b.z? Verify: from=(0,-1) to=(1,0) => -pi/2.
    let cross_y = a.y * b.x - a.x * b.y;
    let dot = a.dot(b);
    let angle = cross_y.atan2(dot);
    if angle > PI {
        angle - 2.0 * PI
    } else if angle < -PI {
        angle + 2.0 * PI
    } else {
        angle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir_after_facing(rotation: Quat) -> Vec3 {
        rotation * Vec3::NEG_Z
    }

    fn dir_at_deg(deg: f32) -> Vec3 {
        Quat::from_rotation_y(deg.to_radians()) * Vec3::NEG_Z
    }

    #[test]
    fn default_facing_looks_along_negative_z() {
        assert!((dir_after_facing(PlayerFacing::default().lower) - Vec3::NEG_Z).length() < 1e-4);
    }

    #[test]
    fn target_rotation_turns_negative_z_toward_a_direction() {
        let target = yaw_from_neg_z(Vec3::X);
        assert!((dir_after_facing(target) - Vec3::X).length() < 1e-4);
    }

    #[test]
    fn opposite_direction_turns_around_y_without_flipping_upside_down() {
        let target = yaw_from_neg_z(Vec3::Z);
        assert!((dir_after_facing(target) - Vec3::Z).length() < 1e-4);
        assert!((target * Vec3::Y - Vec3::Y).length() < 1e-4);
    }

    #[test]
    fn signed_angle_sign_matches_y_rotation() {
        let right_angle = signed_angle_xz(Vec3::NEG_Z, Vec3::X);
        assert!((Quat::from_rotation_y(right_angle) * Vec3::NEG_Z - Vec3::X).length() < 1e-4);

        let left_angle = signed_angle_xz(Vec3::NEG_Z, Vec3::NEG_X);
        assert!((Quat::from_rotation_y(left_angle) * Vec3::NEG_Z - Vec3::NEG_X).length() < 1e-4);
    }

    #[test]
    fn idle_stays_planted_within_max_twist_then_catches_up() {
        let mut twist = TwistConfig::default();
        twist.twist_blend_secs = 0.0;
        let mut facing = PlayerFacing::default();

        step_facing(&mut facing, Some(dir_at_deg(-30.0)), None, true, 0.1, 0.0, &twist);
        assert!(!facing.idle_turning);
        assert!((dir_after_facing(facing.lower) - Vec3::NEG_Z).length() < 1e-4);
        assert!((facing.twist_rad.to_degrees() + 30.0).abs() < 1.0);

        step_facing(&mut facing, Some(dir_at_deg(-70.0)), None, true, 1.0, 0.0, &twist);
        assert!((dir_after_facing(facing.lower) - dir_at_deg(-70.0)).length() < 1e-3);
        assert!(!facing.idle_turning);
        assert!(facing.twist_rad.abs() < 1e-3);
    }

    #[test]
    fn moving_cones_select_forward_sideways_and_backward() {
        let mut twist = TwistConfig::default();
        twist.twist_blend_secs = 0.0;
        let mut facing = PlayerFacing::default();

        step_facing(
            &mut facing,
            Some(dir_at_deg(-30.0)),
            Some(Vec3::NEG_Z),
            true,
            0.1,
            0.0,
            &twist,
        );
        assert_eq!(facing.zone, MoveZone::Forward);
        assert!((dir_after_facing(facing.lower) - Vec3::NEG_Z).length() < 1e-4);
        assert!((facing.twist_rad.to_degrees() + 30.0).abs() < 0.5);

        step_facing(
            &mut facing,
            Some(Vec3::X),
            Some(Vec3::NEG_Z),
            true,
            0.1,
            0.0,
            &twist,
        );
        assert_eq!(facing.zone, MoveZone::Sideways);
        assert!((dir_after_facing(facing.lower) - Vec3::X).length() < 1e-4);
        assert!(facing.twist_rad.abs() < 1e-3);

        step_facing(
            &mut facing,
            Some(dir_at_deg(-160.0)),
            Some(Vec3::NEG_Z),
            true,
            0.1,
            0.0,
            &twist,
        );
        assert_eq!(facing.zone, MoveZone::Backward);
        assert!((dir_after_facing(facing.lower) - Vec3::Z).length() < 1e-4);
        assert!((facing.twist_rad.to_degrees() - 20.0).abs() < 0.5);
    }

    #[test]
    fn cone_boundaries_apply_hysteresis() {
        let twist = TwistConfig::default();
        let mut facing = PlayerFacing::default();

        step_facing(
            &mut facing,
            Some(dir_at_deg(-48.0)),
            Some(Vec3::NEG_Z),
            true,
            0.1,
            0.0,
            &twist,
        );
        assert_eq!(facing.zone, MoveZone::Forward);

        step_facing(
            &mut facing,
            Some(dir_at_deg(-52.0)),
            Some(Vec3::NEG_Z),
            true,
            0.1,
            0.0,
            &twist,
        );
        assert_eq!(facing.zone, MoveZone::Sideways);

        step_facing(
            &mut facing,
            Some(dir_at_deg(-42.0)),
            Some(Vec3::NEG_Z),
            true,
            0.1,
            0.0,
            &twist,
        );
        assert_eq!(facing.zone, MoveZone::Sideways);

        step_facing(
            &mut facing,
            Some(dir_at_deg(-38.0)),
            Some(Vec3::NEG_Z),
            true,
            0.1,
            0.0,
            &twist,
        );
        assert_eq!(facing.zone, MoveZone::Forward);
    }

    #[test]
    fn invalid_aim_retains_previous_facing_and_death_zeroes_twist() {
        let mut twist = TwistConfig::default();
        twist.twist_blend_secs = 0.0;
        let mut facing = PlayerFacing::default();

        step_facing(&mut facing, Some(Vec3::X), Some(Vec3::X), true, 0.1, 0.0, &twist);
        let saved_lower = facing.lower;
        let saved_upper = facing.upper;

        step_facing(&mut facing, None, None, true, 0.1, 0.0, &twist);
        assert!((facing.lower.dot(saved_lower)).abs() > 0.999);
        assert!((facing.upper.dot(saved_upper)).abs() > 0.999);

        facing.twist_rad = 0.5;
        step_facing(&mut facing, Some(Vec3::NEG_Z), None, false, 0.1, 0.0, &twist);
        assert_eq!(facing.twist_rad, 0.0);
        assert!(!facing.idle_turning);
    }
}
