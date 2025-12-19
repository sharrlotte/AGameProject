use bevy::prelude::*;

use crate::content::turret::base::Turret;

pub fn system_turret_tracking(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &GlobalTransform, &Turret)>,
    transforms: Query<&GlobalTransform>,
) {
    for (mut transform, global_transform, targeter) in &mut query {
        let mut target_pos: Option<Vec2> = None;
        if let Some(shoot_at) = targeter.shoot_at {
            target_pos = Some(shoot_at);
        } else if let Some(target) = targeter.target {
            if let Ok(pos) = transforms.get(target) {
                target_pos = Some(pos.translation().truncate());
            }
        }

        if let Some(shoot_at_pos) = target_pos {
            let current_pos = global_transform.translation().truncate();
            let diff = shoot_at_pos - current_pos;
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

            let max_rot = targeter.turn_speed * time.delta_secs();

            // Clamp rotation to max_rot, but don't overshoot
            let rot_amount = angle_error.clamp(-max_rot, max_rot);

            transform.rotate_z(rot_amount);
        }
    }
}

pub fn draw_turret_ranges(mut gizmos: Gizmos, query: Query<(&GlobalTransform, &Turret)>) {
    for (global_transform, tracker) in &query {
        if tracker.range > 0.0 {
            gizmos
                .circle_2d(
                    global_transform.translation().truncate(),
                    tracker.range,
                    Color::srgb(0.0, 1.0, 1.0).with_alpha(0.3),
                )
                .resolution(64);

            gizmos.line_2d(
                global_transform.translation().truncate(),
                global_transform.translation().truncate()
                    + (global_transform.to_scale_rotation_translation().1
                        * Vec3::Y
                        * tracker.range)
                        .truncate(),
                Color::srgb(0.5, 1.0, 1.0).with_alpha(0.3),
            );
        }
    }
}
