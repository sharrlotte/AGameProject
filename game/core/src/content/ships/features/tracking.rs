use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::content::ships::features::FeatureState;

pub struct TrackingFeaturePlugin;

#[derive(Component)]
pub struct Rotatable {
    pub speed: f32,
}

#[derive(Component)]
pub struct Tracker {
    pub range: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct RotatableFeature {
    pub turn_speed: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct TrackerFeature {
    pub range: f32,
    pub fov: f32,
}

impl Plugin for TrackingFeaturePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (system_tracker_tracking, draw_tracker_ranges));
    }
}

pub fn system_tracker_tracking(
    time: Res<Time>,
    state: Res<FeatureState>,
    mut query: Query<(
        &mut Transform,
        &GlobalTransform,
        &Tracker,
        Option<&Rotatable>,
    )>,
) {
    if let Some(target_pos) = state.cursor_world_pos {
        for (mut transform, global_transform, _tracker, rotatable) in &mut query {
            let current_pos = global_transform.translation().truncate();
            let diff = target_pos - current_pos;
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

            // Determine max rotation this frame
            let speed = if let Some(r) = rotatable {
                r.speed
            } else {
                2.0 // Default slower speed
            };

            let max_rot = speed * time.delta_secs();

            // Clamp rotation to max_rot, but don't overshoot
            let rot_amount = angle_error.clamp(-max_rot, max_rot);

            transform.rotate_z(rot_amount);
        }
    }
}

fn draw_tracker_ranges(mut gizmos: Gizmos, query: Query<(&GlobalTransform, &Tracker)>) {
    for (global_transform, tracker) in &query {
        if tracker.range > 0.0 {
            gizmos
                .circle_2d(
                    global_transform.translation().truncate(),
                    tracker.range,
                    Color::srgb(0.0, 1.0, 1.0).with_alpha(0.3),
                )
                .resolution(64);
        }
    }
}
