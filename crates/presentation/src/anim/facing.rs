use std::f32::consts::{PI, TAU};

use bevy::prelude::*;
use data::GameState;
use world::{AimDirection, Health};

use crate::anim::PlayerVisualRoot;
use crate::anim::catalog::AnimCatalog;

/// Yaw-only always-aim facing state kept on the visual root.
///
/// The body always turns toward the cursor, so there is no separate lower and
/// upper yaw and no residual spine twist to distribute.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct PlayerFacing {
    pub yaw: f32,
}

impl PlayerFacing {
    pub fn rotation(self) -> Quat {
        Quat::from_rotation_y(self.yaw)
    }

    pub fn forward(self) -> Vec3 {
        self.rotation() * Vec3::NEG_Z
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

/// Smooths the visual root toward the cursor yaw and writes it to the transform.
pub fn sync_player_facing(
    time: Res<Time>,
    state: Option<Res<State<GameState>>>,
    catalog: Res<AnimCatalog>,
    survivors: Query<(&AimDirection, &Health)>,
    mut roots: Query<(&ChildOf, &mut Transform, &mut PlayerFacing), With<PlayerVisualRoot>>,
) {
    let dt = time.delta_secs();
    let game_over = state
        .as_deref()
        .map(|s| *s.get() == GameState::GameOver)
        .unwrap_or(false);

    for (child_of, mut transform, mut facing) in &mut roots {
        let Ok((aim, health)) = survivors.get(child_of.parent()) else {
            continue;
        };

        let alive = health.current > 0.0 && !game_over;
        step_facing(
            &mut facing,
            Some(aim.0),
            alive,
            dt,
            catalog.facing.blend_secs,
        );
        transform.rotation = facing.rotation();
    }
}

/// Pure always-aim step: shortest-arc exponential smoothing toward the aim yaw.
///
/// Invalid aim or a dead player keeps the previous yaw untouched.
pub fn step_facing(
    facing: &mut PlayerFacing,
    aim_dir: Option<Vec3>,
    alive: bool,
    dt: f32,
    blend_secs: f32,
) {
    if !alive {
        return;
    }
    let Some(target) = aim_dir.and_then(horizontal_yaw) else {
        return;
    };

    let alpha = smoothing_alpha(dt, blend_secs);
    facing.yaw = wrap_angle(facing.yaw + wrap_angle(target - facing.yaw) * alpha);
}

/// Frame-rate independent exponential smoothing factor.
pub fn smoothing_alpha(dt: f32, blend_secs: f32) -> f32 {
    if blend_secs <= 0.0 || dt <= 0.0 {
        return 1.0;
    }
    (1.0 - (-dt / blend_secs).exp()).clamp(0.0, 1.0)
}

/// Wraps an angle into `[-pi, pi)`.
pub fn wrap_angle(angle: f32) -> f32 {
    let wrapped = (angle + PI).rem_euclid(TAU) - PI;
    if wrapped >= PI {
        wrapped - TAU
    } else {
        wrapped
    }
}

/// Pure Y-axis yaw from `-Z` toward a horizontal direction.
pub fn horizontal_yaw(dir: Vec3) -> Option<f32> {
    let flat = Vec3::new(dir.x, 0.0, dir.z);
    if flat.length_squared() <= 1e-6 {
        return None;
    }
    let norm = flat.normalize();
    Some(wrap_angle((-norm.x).atan2(-norm.z)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir_at_deg(deg: f32) -> Vec3 {
        Quat::from_rotation_y(deg.to_radians()) * Vec3::NEG_Z
    }

    fn facing_deg(facing: PlayerFacing) -> f32 {
        facing.yaw.to_degrees()
    }

    #[test]
    fn default_facing_looks_along_negative_z() {
        assert!((PlayerFacing::default().forward() - Vec3::NEG_Z).length() < 1e-4);
    }

    #[test]
    fn smoothing_converges_and_never_overshoots() {
        let mut facing = PlayerFacing::default();
        let target = dir_at_deg(90.0);

        let mut previous = facing.yaw;
        for _ in 0..120 {
            step_facing(&mut facing, Some(target), true, 1.0 / 60.0, 0.12);
            assert!(facing.yaw <= 90.0f32.to_radians() + 1e-5);
            assert!(facing.yaw >= previous - 1e-5);
            previous = facing.yaw;
        }
        assert!((facing_deg(facing) - 90.0).abs() < 0.5);
    }

    #[test]
    fn smoothing_is_frame_rate_independent() {
        let target = dir_at_deg(-120.0);

        let mut coarse = PlayerFacing::default();
        step_facing(&mut coarse, Some(target), true, 0.1, 0.12);

        let mut fine = PlayerFacing::default();
        for _ in 0..10 {
            step_facing(&mut fine, Some(target), true, 0.01, 0.12);
        }

        assert!((coarse.yaw - fine.yaw).abs() < 1e-4);
    }

    #[test]
    fn smoothing_takes_the_short_arc_across_the_seam() {
        let mut facing = PlayerFacing {
            yaw: 179.0f32.to_radians(),
        };
        let target = dir_at_deg(-179.0);

        step_facing(&mut facing, Some(target), true, 1.0, 10.0);
        let moved = wrap_angle(facing.yaw - 179.0f32.to_radians()).to_degrees();
        assert!(
            moved > 0.0 && moved < 5.0,
            "short arc moved {moved} degrees"
        );
    }

    #[test]
    fn invalid_aim_keeps_previous_yaw() {
        let mut facing = PlayerFacing { yaw: 0.7 };
        step_facing(&mut facing, None, true, 0.1, 0.12);
        assert_eq!(facing.yaw, 0.7);

        step_facing(&mut facing, Some(Vec3::Y), true, 0.1, 0.12);
        assert_eq!(facing.yaw, 0.7);
    }

    #[test]
    fn death_suppresses_facing_changes() {
        let mut facing = PlayerFacing { yaw: -0.4 };
        step_facing(&mut facing, Some(Vec3::X), false, 0.1, 0.0);
        assert_eq!(facing.yaw, -0.4);
    }

    #[test]
    fn zero_blend_snaps_to_target() {
        let mut facing = PlayerFacing::default();
        step_facing(&mut facing, Some(Vec3::X), true, 0.016, 0.0);
        assert!((facing.forward() - Vec3::X).length() < 1e-4);
    }

    #[test]
    fn wrap_angle_maps_into_half_open_turn() {
        assert!((wrap_angle(PI) + PI).abs() < 1e-4);
        assert!((wrap_angle(3.0 * PI) + PI).abs() < 1e-4);
        assert!((wrap_angle(-PI) + PI).abs() < 1e-4);
        assert!(wrap_angle(TAU).abs() < 1e-4);
        assert!((wrap_angle(0.25) - 0.25).abs() < 1e-6);
    }
}
