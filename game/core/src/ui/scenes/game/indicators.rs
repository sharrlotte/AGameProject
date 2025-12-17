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
    ships: Query<(&GlobalTransform, &Team), With<Ship>>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
    window: Single<&Window>,
) {
    let (camera, camera_transform) = *camera;
    let window_size = Vec2::new(window.width(), window.height());

    for (ship_transform, team) in &ships {
        let ship_pos = ship_transform.translation();

        // Project to screen to check visibility and clamp
        if let Ok(viewport_pos) = camera.world_to_viewport(camera_transform, ship_pos) {
            let margin = 30.0;
            let clamped_x = viewport_pos.x.clamp(margin, window_size.x - margin);
            let clamped_y = viewport_pos.y.clamp(margin, window_size.y - margin);

            let clamped_viewport_pos = Vec2::new(clamped_x, clamped_y);

            // Convert back to world space for drawing Gizmos
            if let Ok(world_pos) =
                camera.viewport_to_world_2d(camera_transform, clamped_viewport_pos)
            {
                let color = if team.id == 0 {
                    Color::srgb(0.0, 1.0, 0.0) // Green
                } else {
                    Color::srgb(1.0, 0.0, 0.0) // Red
                };

                // Triangle pointing down
                let size = 30.0; // World units

                // Tip at the location
                let p1 = world_pos;
                // Top Left
                let p2 = world_pos + Vec2::new(-size * 0.5, size);
                // Top Right
                let p3 = world_pos + Vec2::new(size * 0.5, size);

                gizmos.linestrip_2d(vec![p1, p2, p3, p1], color);
            }
        }
    }
}
