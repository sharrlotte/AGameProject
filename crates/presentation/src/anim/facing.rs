use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;
use world::{AimDirection, Stamina};

use crate::anim::PlayerVisualRoot;
use crate::anim::catalog::AnimCatalog;
use crate::config::VisualsConfig;

/// Last committed visual facing, kept on the player visual root.
#[derive(Component, Debug, Clone, Copy)]
pub struct PlayerFacing {
    pub rotation: Quat,
}

impl Default for PlayerFacing {
    fn default() -> Self {
        Self {
            rotation: Quat::IDENTITY,
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

/// Faces the aim direction by default and horizontal velocity while sprinting.
///
/// Sprint wins even while aim is held. When no aim direction is available the
/// previous facing is retained, so the body never snaps to a default axis.
pub fn sync_player_facing(
    time: Res<Time>,
    catalog: Res<AnimCatalog>,
    cfg: Res<VisualsConfig>,
    survivors: Query<(&AimDirection, &Stamina, &Velocity)>,
    mut roots: Query<(&ChildOf, &mut Transform, &mut PlayerFacing), With<PlayerVisualRoot>>,
) {
    let dt = time.delta_secs();
    let threshold_sq = cfg.player.move_speed_threshold.max(0.0).powi(2);
    let blend = catalog.facing_blend_secs.max(0.0);
    let alpha = if blend <= 0.0 || dt <= 0.0 {
        1.0
    } else {
        (1.0 - (-dt / blend).exp()).clamp(0.0, 1.0)
    };

    for (child_of, mut transform, mut facing) in &mut roots {
        let Ok((aim, stamina, velocity)) = survivors.get(child_of.parent()) else {
            continue;
        };

        let horizontal = Vec3::new(velocity.linear.x, 0.0, velocity.linear.z);
        let aim_dir = if aim.0.length_squared() > 1e-4 {
            Some(aim.0.normalize())
        } else {
            None
        };

        let dir = if stamina.is_sprinting && horizontal.length_squared() > threshold_sq {
            horizontal.normalize()
        } else if let Some(dir) = aim_dir {
            dir
        } else {
            facing.rotation * Vec3::NEG_Z
        };

        if dir.length_squared() <= 1e-4 {
            continue;
        }

        let target = Quat::from_rotation_arc(Vec3::NEG_Z, dir.normalize());
        facing.rotation = facing.rotation.slerp(target, alpha);
        transform.rotation = facing.rotation;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir_after_facing(rotation: Quat) -> Vec3 {
        rotation * Vec3::NEG_Z
    }

    #[test]
    fn default_facing_looks_along_negative_z() {
        assert!((dir_after_facing(PlayerFacing::default().rotation) - Vec3::NEG_Z).length() < 1e-4);
    }

    #[test]
    fn target_rotation_turns_negative_z_toward_a_direction() {
        let target = Quat::from_rotation_arc(Vec3::NEG_Z, Vec3::X);
        assert!((dir_after_facing(target) - Vec3::X).length() < 1e-4);
    }
}
