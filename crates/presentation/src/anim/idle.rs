use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;

use data::{GameState, Survivor};
use world::Health;

use crate::anim::rig::PlayerRig;

/// Procedurally relaxes the idle pose (shoulders, torso) after IK, before transform
/// propagation. Reduces the spread-arm silhouette of the idle animation clip.
pub fn relax_idle_pose(
    state: Res<State<GameState>>,
    survivors: Query<(&Velocity, &Health), With<Survivor>>,
    rigs: Query<&PlayerRig>,
    mut transforms: Query<&mut Transform>,
) {
    if !matches!(state.get(), GameState::Playing | GameState::GameOver) {
        return;
    }
    let move_threshold = 0.25;

    for rig in &rigs {
        let Ok((velocity, health)) = survivors.get(rig.survivor) else {
            continue;
        };
        if health.current <= 0.0 {
            continue;
        }
        let speed = Vec3::new(velocity.linear.x, 0.0, velocity.linear.z).length();
        if speed >= move_threshold {
            continue;
        }

        // Relax shoulders: reduce Z abduction.
        for shoulder in [rig.arms.right.shoulder, rig.arms.left.shoulder]
            .into_iter()
            .flatten()
        {
            if let Ok(mut transform) = transforms.get_mut(shoulder) {
                let (x, y, z) = transform.rotation.to_euler(EulerRot::XYZ);
                transform.rotation = Quat::from_euler(EulerRot::XYZ, x, y, z * 0.6);
            }
        }

        // Relax torso: reduce X rotation by 50%.
        if let Some(torso) = rig.arms.torso
            && let Ok(mut transform) = transforms.get_mut(torso)
        {
            let (x, y, z) = transform.rotation.to_euler(EulerRot::XYZ);
            transform.rotation = Quat::from_euler(EulerRot::XYZ, x * 0.5, y, z);
        }
    }
}
