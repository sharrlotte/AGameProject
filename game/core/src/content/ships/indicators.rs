use std::f32::consts::PI;

use crate::content::structures::ship::Ship;
use crate::content::team::Team;
use bevy::prelude::*;

pub struct IndicatorsPlugin;

impl Plugin for IndicatorsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, draw_indicators);
    }
}

fn draw_indicators(
    mut gizmos: Gizmos,
    ships: Query<(&GlobalTransform, &Team, &Ship)>,
    camera: Single<(&Camera, &GlobalTransform, &Projection), With<Camera2d>>,
    window: Single<&Window>,
) {
    let (camera, camera_transform, projection) = *camera;
    // max(32) prevent crash when window size is too small, idk how
    let window_size = Vec2::new(window.width().max(32.0), window.height().max(32.0));

    for (ship_transform, team, ship) in &ships {
        let ship_pos = ship_transform.transform_point(ship.center.extend(0.0));

        // Project to screen to check visibility and clamp
        if let Ok(viewport_pos) = camera.world_to_viewport(camera_transform, ship_pos) {
            let scale = if let Projection::Orthographic(ortho) = &*projection {
                ortho.scale
            } else {
                1.0
            };
            let margin = 30.0;
            let clamped_x = viewport_pos.x.clamp(margin, window_size.x - margin);
            let clamped_y = viewport_pos.y.clamp(margin, window_size.y - margin);

            let clamped_viewport_pos = Vec2::new(clamped_x, clamped_y);

            // Convert back to world space for drawing Gizmos
            if let Ok(world_pos) =
                camera.viewport_to_world_2d(camera_transform, clamped_viewport_pos)
            {
                let color = if team.is_player() {
                    Color::srgb(0.0, 1.1, 0.0) // Green
                } else {
                    Color::srgb(1.1, 0.0, 0.0) // Red
                };

                // Triangle pointing toward the target
                let size = 30.0 * scale; // 1.5 times bigger

                let diff = ship_pos.truncate() - world_pos;
                // Only draw if we are somewhat far from the target (avoid drawing on top)
                let angle: f32 = if diff.length_squared() > 0.5 {
                    diff.y.atan2(diff.x)
                } else {
                    -PI / 2.0
                };

                let rotation = Quat::from_rotation_z(angle);

                // Local points: Tip at (0,0), Base at -size
                let p1 = Vec2::ZERO; // Tip
                let p2 = Vec2::new(-size, size * 0.5); // Top Back
                let p3 = Vec2::new(-size, -size * 0.5); // Bottom Back

                // Rotate and Translate
                let transform_point =
                    |p: Vec2| -> Vec2 { world_pos + (rotation * p.extend(0.0)).truncate() };

                let v1 = transform_point(p1);
                let v2 = transform_point(p2);
                let v3 = transform_point(p3);

                // Draw outline (Gizmos don't support solid fill natively yet)
                gizmos.linestrip_2d(vec![v1, v2, v3, v1], color);
            }
        }
    }
}
