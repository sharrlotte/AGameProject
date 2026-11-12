use std::f32::consts::PI;

use bevy::prelude::*;

use crate::anim::blend::approach;
use crate::anim::weapon::catalog::{ReloadConfig, WeaponPose, WeaponProfile};

/// Socket entities resolved once from the mounted weapon glTF.
#[derive(Debug, Clone, Copy)]
pub struct ResolvedSockets {
    pub grip: Entity,
    pub foregrip: Entity,
    pub magazine: Option<Entity>,
    pub muzzle: Option<Entity>,
}

/// Runtime mount cached on the weapon anchor entity.
#[derive(Component, Debug, Clone)]
pub struct WeaponMount {
    pub survivor: Entity,
    pub root: Entity,
    pub profile: String,
    pub scene: Entity,
    pub sockets: ResolvedSockets,
    pub magazine_rest: Transform,
    pub resolved: bool,
}

/// Mutable aim, recoil, sway, and swap state on the anchor.
#[derive(Component, Debug, Clone, Default)]
pub struct WeaponPoseState {
    pub aim: f32,
    pub kick_pitch: f32,
    pub kick_back: f32,
    pub sway: f32,
    pub swap_elapsed: f32,
    pub swap_duration: f32,
}

/// Marks the chest-height weapon anchor under `PlayerVisualRoot`.
#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerWeaponAnchor {
    pub root: Entity,
    pub survivor: Entity,
}

/// Procedural reload offsets derived from normalized progress.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ReloadOffset {
    pub tilt: f32,
    pub drop: f32,
    pub mag_drop: f32,
}

pub fn euler_deg(rotation_deg: [f32; 3]) -> Quat {
    Quat::from_euler(
        EulerRot::XYZ,
        rotation_deg[0].to_radians(),
        rotation_deg[1].to_radians(),
        rotation_deg[2].to_radians(),
    )
    .normalize()
}

/// Blends the relaxed and aiming weapon offsets.
pub fn weapon_pose(profile: &WeaponProfile, aim: f32) -> Transform {
    let t = aim.clamp(0.0, 1.0);
    TransformPose::blend(&profile.relaxed, &profile.aim, t).into_transform()
}

#[derive(Debug, Clone, Copy)]
struct TransformPose {
    translation: Vec3,
    rotation: Quat,
}

impl TransformPose {
    fn blend(relaxed: &WeaponPose, aim: &WeaponPose, t: f32) -> Self {
        let translation = Vec3::from_array(relaxed.offset).lerp(Vec3::from_array(aim.offset), t);
        let rotation = euler_deg(relaxed.rotation_deg)
            .slerp(euler_deg(aim.rotation_deg), t)
            .normalize();
        Self {
            translation,
            rotation,
        }
    }

    fn into_transform(self) -> Transform {
        Transform::from_translation(self.translation).with_rotation(self.rotation)
    }
}

/// Reload tilt and drops peak at the middle of the reload and vanish at both ends.
pub fn reload_offset(progress: Option<f32>, cfg: &ReloadConfig) -> ReloadOffset {
    let Some(progress) = progress.map(|value| value.clamp(0.0, 1.0)) else {
        return ReloadOffset::default();
    };
    let curve = (PI * progress).sin().max(0.0);
    ReloadOffset {
        tilt: cfg.tilt_deg.to_radians() * curve,
        drop: cfg.drop * curve,
        mag_drop: cfg.mag_drop * curve,
    }
}

/// Lowering curve for a weapon swap: zero at both ends, maximum in the middle.
pub fn swap_curve(elapsed: f32, duration: f32) -> f32 {
    if !duration.is_finite() || duration <= 0.0 || !elapsed.is_finite() {
        return 0.0;
    }
    let t = (elapsed / duration).clamp(0.0, 1.0);
    (PI * t).sin().max(0.0)
}

/// Adds one shot impulse, clamped to the profile's recoil limits.
pub fn add_kick(state: &mut WeaponPoseState, profile: &WeaponProfile) {
    let pitch = profile.recoil.pitch_deg.to_radians();
    let max_pitch = profile.recoil.max_deg.to_radians();
    state.kick_pitch = (state.kick_pitch + pitch).min(max_pitch);
    state.kick_back = (state.kick_back + profile.recoil.back).min(profile.recoil.max_back);
}

/// Critically approaches zero using the profile's recovery time.
pub fn decay_kick(state: &mut WeaponPoseState, dt: f32, profile: &WeaponProfile) {
    let recover = profile.recoil.recover_secs;
    state.kick_pitch = approach(state.kick_pitch, 0.0, dt, recover);
    state.kick_back = approach(state.kick_back, 0.0, dt, recover);
}

/// Composes the final weapon transform relative to its anchor.
pub fn compose_weapon(
    profile: &WeaponProfile,
    state: &WeaponPoseState,
    reload: ReloadOffset,
    swap: f32,
) -> Transform {
    let mut pose = weapon_pose(profile, state.aim);
    let sway = sway_offset(profile, state);
    pose.translation += sway.translation;
    pose.rotation = (sway.rotation * pose.rotation).normalize();
    pose.translation.y -= reload.drop + profile.swap.drop * swap;
    pose.translation.z += state.kick_back;
    pose.rotation =
        (Quat::from_rotation_x(state.kick_pitch - reload.tilt) * pose.rotation).normalize();
    pose
}

/// Low-frequency bob around the composed pose, damped while aiming.
pub fn sway_offset(profile: &WeaponProfile, state: &WeaponPoseState) -> Transform {
    let cfg = &profile.sway;
    let weight = 1.0 - 0.6 * state.aim.clamp(0.0, 1.0);
    let phase = state.sway;
    let translation = Vec3::new(
        cfg.amplitude * weight * phase.sin(),
        cfg.amplitude * weight * (phase * 2.0).sin() * 0.5,
        0.0,
    );
    let roll = cfg.roll_deg.to_radians() * weight * (phase * 0.5).sin();
    Transform::from_translation(translation).with_rotation(Quat::from_rotation_z(roll))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile() -> WeaponProfile {
        WeaponProfile::default()
    }

    #[test]
    fn weapon_pose_blends_relaxed_and_aim_offsets() {
        let profile = profile();
        let relaxed = weapon_pose(&profile, 0.0);
        let aim = weapon_pose(&profile, 1.0);
        let halfway = weapon_pose(&profile, 0.5);

        assert!((relaxed.translation - Vec3::from_array(profile.relaxed.offset)).length() < 1e-4);
        assert!((aim.translation - Vec3::from_array(profile.aim.offset)).length() < 1e-4);
        assert!(
            (halfway.translation - relaxed.translation.lerp(aim.translation, 0.5)).length() < 1e-4
        );
    }

    #[test]
    fn reload_offset_peaks_midway_and_clears_at_ends() {
        let cfg = profile().reload;
        assert_eq!(reload_offset(None, &cfg), ReloadOffset::default());
        assert!(reload_offset(Some(0.0), &cfg).tilt.abs() < 1e-4);
        assert!(reload_offset(Some(1.0), &cfg).tilt.abs() < 1e-4);

        let middle = reload_offset(Some(0.5), &cfg);
        assert!((middle.tilt - cfg.tilt_deg.to_radians()).abs() < 1e-3);
        assert!((middle.drop - cfg.drop).abs() < 1e-3);
        assert!((middle.mag_drop - cfg.mag_drop).abs() < 1e-3);
    }

    #[test]
    fn swap_curve_rises_and_falls() {
        assert_eq!(swap_curve(0.0, 0.3), 0.0);
        assert!((swap_curve(0.15, 0.3) - 1.0).abs() < 1e-3);
        assert!(swap_curve(0.3, 0.3).abs() < 1e-3);
        assert_eq!(swap_curve(0.1, 0.0), 0.0);
    }

    #[test]
    fn kick_clamps_and_decays() {
        let profile = profile();
        let mut state = WeaponPoseState::default();
        for _ in 0..20 {
            add_kick(&mut state, &profile);
        }
        assert!((state.kick_pitch - profile.recoil.max_deg.to_radians()).abs() < 1e-4);
        assert!((state.kick_back - profile.recoil.max_back).abs() < 1e-4);

        for _ in 0..120 {
            decay_kick(&mut state, 1.0 / 60.0, &profile);
        }
        assert!(state.kick_pitch < 0.01);
        assert!(state.kick_back < 0.01);
    }

    #[test]
    fn compose_weapon_applies_reload_and_swap_drop() {
        let profile = profile();
        let mut state = WeaponPoseState::default();
        let base = compose_weapon(&profile, &state, ReloadOffset::default(), 0.0);
        let reload = reload_offset(Some(0.5), &profile.reload);
        let dropped = compose_weapon(&profile, &state, reload, 1.0);
        let expected = reload.drop + profile.swap.drop;
        assert!((base.translation.y - dropped.translation.y - expected).abs() < 1e-3);

        state.kick_back = 0.05;
        let kicked = compose_weapon(&profile, &state, ReloadOffset::default(), 0.0);
        assert!((kicked.translation.z - base.translation.z - 0.05).abs() < 1e-4);
    }

    #[test]
    fn sway_is_bounded_and_damped_while_aiming() {
        let profile = profile();
        let mut state = WeaponPoseState::default();
        let rest = sway_offset(&profile, &state);
        assert!(rest.translation.length() < 1e-5);

        state.sway = std::f32::consts::FRAC_PI_2;
        let relaxed = sway_offset(&profile, &state);
        assert!((relaxed.translation.x - profile.sway.amplitude).abs() < 1e-5);

        state.aim = 1.0;
        let aiming = sway_offset(&profile, &state);
        assert!(aiming.translation.x.abs() < relaxed.translation.x.abs());
        assert!(aiming.translation.length() <= profile.sway.amplitude);
    }
}
