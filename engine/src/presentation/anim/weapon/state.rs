use std::f32::consts::PI;

use bevy::prelude::*;

use crate::config::{WeaponDefinition, WeaponPose, WeaponReload};
use crate::presentation::anim::blend::approach;

/// Presentation-scoped equipped weapon runtime holding the active weapon definition.
#[derive(Component, Debug, Clone)]
pub struct PlayerWeaponRuntime {
    pub weapon: WeaponDefinition,
}

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
    /// The weapon's placement inside the anchor, without the model transform.
    ///
    /// Grip targets are expressed in the weapon's own space, so the IK needs
    /// this frame rather than the weapon's world transform.
    pub frame: Transform,
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
pub fn weapon_pose(weapon: &WeaponDefinition, aim: f32) -> Transform {
    let t = aim.clamp(0.0, 1.0);
    TransformPose::blend(&weapon.relaxed, &weapon.aim, t).into_transform()
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
pub fn reload_offset(progress: Option<f32>, cfg: &WeaponReload) -> ReloadOffset {
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

/// Adds one shot impulse, clamped to the weapon's recoil limits.
pub fn add_kick(state: &mut WeaponPoseState, weapon: &WeaponDefinition) {
    let recoil = &weapon.recoil;
    let pitch = recoil.pitch_deg.to_radians();
    let max_pitch = recoil.max_deg.to_radians();
    state.kick_pitch = (state.kick_pitch + pitch).min(max_pitch);
    state.kick_back = (state.kick_back + recoil.back).min(recoil.max_back);
}

/// Critically approaches zero using the weapon's recovery time.
pub fn decay_kick(state: &mut WeaponPoseState, dt: f32, weapon: &WeaponDefinition) {
    let recover = weapon.recoil.recover_secs;
    state.kick_pitch = approach(state.kick_pitch, 0.0, dt, recover);
    state.kick_back = approach(state.kick_back, 0.0, dt, recover);
}

/// Composes the final weapon transform relative to its anchor.
pub fn compose_weapon(
    weapon: &WeaponDefinition,
    state: &WeaponPoseState,
    reload: ReloadOffset,
    swap: f32,
) -> Transform {
    let mut pose = weapon_pose(weapon, state.aim);
    let sway = sway_offset(weapon, state);
    pose.translation += sway.translation;
    pose.rotation = (sway.rotation * pose.rotation).normalize();
    pose.translation.y -= reload.drop + weapon.swap.drop * swap;
    pose.translation.z += state.kick_back;
    pose.rotation = (Quat::from_rotation_x(state.kick_pitch - reload.tilt) * pose.rotation).normalize();
    pose
}

/// Low-frequency bob around the composed pose, damped while aiming.
pub fn sway_offset(weapon: &WeaponDefinition, state: &WeaponPoseState) -> Transform {
    let cfg = &weapon.sway;
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

    fn weapon() -> WeaponDefinition {
        WeaponDefinition {
            id: "m16".to_string(),
            name: "M16".to_string(),
            muzzle_offset: 0.6,
            sprint_spread_multiplier: 1.65,
            swap_cooldown_cap: 0.12,
            stance_hold_secs: 0.15,
            full_auto: true,
            fire_interval: 0.1,
            pellets: 1,
            damage: 24.0,
            spread_deg: 3.0,
            projectile_speed: 42.0,
            projectile_radius: 0.1,
            projectile_lifetime: 1.2,
            knockback: 3.5,
            damage_kind: crate::common::DamageKind::Ballistic,
            mag_size: 30,
            reload_secs: 1.8,
            relaxed: crate::config::WeaponPose {
                offset: [0.16, -0.19, -0.51],
                rotation_deg: [-32.0, 90.0, 0.0],
            },
            aim: crate::config::WeaponPose {
                offset: [0.14, -0.03, -0.5],
                rotation_deg: [0.0, 0.0, 0.0],
            },
            recoil: crate::config::WeaponRecoil {
                pitch_deg: 4.0,
                back: 0.04,
                recover_secs: 0.12,
                max_deg: 12.0,
                max_back: 0.12,
            },
            reload: crate::config::WeaponReload {
                tilt_deg: 22.0,
                drop: 0.08,
                mag_drop: 0.12,
                hand_return: 0.7,
            },
            sway: crate::config::WeaponSway {
                amplitude: 0.012,
                roll_deg: 1.5,
                frequency: 1.4,
            },
            swap: crate::config::WeaponSwap {
                lower_secs: 0.12,
                raise_secs: 0.18,
                drop: 0.2,
            },
            model: "models/weapons/m16.glb".to_string(),
            model_space: crate::config::ModelSpace::ForwardNegYUpZ,
            model_rotation_deg: [0.0; 3],
            model_scale: 1.4,
            sockets: crate::config::WeaponSockets::default(),
            socket_overrides: Vec::new(),
            grip: None,
        }
    }

    #[test]
    fn weapon_pose_blends_relaxed_and_aim_offsets() {
        let weapon = weapon();
        let relaxed = weapon_pose(&weapon, 0.0);
        let aim = weapon_pose(&weapon, 1.0);
        let halfway = weapon_pose(&weapon, 0.5);

        assert!((relaxed.translation - Vec3::from_array(weapon.relaxed.offset)).length() < 1e-4);
        assert!((aim.translation - Vec3::from_array(weapon.aim.offset)).length() < 1e-4);
        let expected = relaxed.translation + (aim.translation - relaxed.translation) * 0.5;
        assert!((halfway.translation - expected).length() < 1e-4);
    }

    #[test]
    fn reload_offset_peaks_midway_and_clears_at_ends() {
        let weapon = weapon();
        let cfg = weapon.reload;
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
        let weapon = weapon();
        let mut state = WeaponPoseState::default();
        for _ in 0..20 {
            add_kick(&mut state, &weapon);
        }
        assert!((state.kick_pitch - weapon.recoil.max_deg.to_radians()).abs() < 1e-4);
        assert!((state.kick_back - weapon.recoil.max_back).abs() < 1e-4);

        for _ in 0..120 {
            decay_kick(&mut state, 1.0 / 60.0, &weapon);
        }
        assert!(state.kick_pitch < 0.01);
        assert!(state.kick_back < 0.01);
    }

    #[test]
    fn compose_weapon_applies_reload_and_swap_drop() {
        let weapon = weapon();
        let mut state = WeaponPoseState::default();
        let base = compose_weapon(&weapon, &state, ReloadOffset::default(), 0.0);
        let reload = reload_offset(Some(0.5), &weapon.reload);
        let dropped = compose_weapon(&weapon, &state, reload, 1.0);
        let expected = reload.drop + weapon.swap.drop;
        assert!((base.translation.y - dropped.translation.y - expected).abs() < 1e-3);

        state.kick_back = 0.05;
        let kicked = compose_weapon(&weapon, &state, ReloadOffset::default(), 0.0);
        assert!((kicked.translation.z - base.translation.z - 0.05).abs() < 1e-4);
    }

    #[test]
    fn sway_is_bounded_and_damped_while_aiming() {
        let weapon = weapon();
        let mut state = WeaponPoseState::default();
        let rest = sway_offset(&weapon, &state);
        assert!(rest.translation.length() < 1e-5);

        state.sway = std::f32::consts::FRAC_PI_2;
        let relaxed = sway_offset(&weapon, &state);
        assert!((relaxed.translation.x - weapon.sway.amplitude).abs() < 1e-5);

        state.aim = 1.0;
        let aiming = sway_offset(&weapon, &state);
        assert!(aiming.translation.x.abs() < relaxed.translation.x.abs());
        assert!(aiming.translation.length() <= weapon.sway.amplitude);
    }
}
