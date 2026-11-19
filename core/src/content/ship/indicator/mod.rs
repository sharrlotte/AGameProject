use std::f32::consts::PI;

use crate::content::ship::Ship;
use crate::content::ship::ai::TacticalBoard;
use crate::content::ship::vision::visible_to_player;
use crate::content::team::Team;
use bevy::prelude::*;

pub(super) fn draw_indicators(
    mut gizmos: Gizmos,
    board: Res<TacticalBoard>,
    ships: Query<(Entity, &GlobalTransform, &Team, &Ship)>,
    camera: Single<(&Camera, &GlobalTransform, &Projection), With<Camera2d>>,
    window: Single<&Window>,
) {
    let (camera, camera_transform, projection) = *camera;
    let window_size = Vec2::new(window.width(), window.height());

    for (entity, ship_transform, team, ship) in &ships {
        if !visible_to_player(&board, team, entity) {
            continue;
        }

        let ship_position = ship_transform.transform_point(ship.core_position.extend(0.0));

        if let Ok(viewport_position) = camera.world_to_viewport(camera_transform, ship_position) {
            let scale = if let Projection::Orthographic(ortho) = &*projection {
                ortho.scale
            } else {
                1.0
            };
            let margin = 30.0;
            let clamped_x = viewport_position
                .x
                .clamp(margin, (window_size.x - margin).max(margin + 1.0));
            let clamped_y = viewport_position
                .y
                .clamp(margin, (window_size.y - margin).max(margin + 1.0));

            let clamped_viewport_position = Vec2::new(clamped_x, clamped_y);

            if let Ok(world_position) =
                camera.viewport_to_world_2d(camera_transform, clamped_viewport_position)
            {
                let color = team.color(1.0);
                let size = 45.0 * scale.sqrt();

                let diff = ship_position.truncate() - world_position;
                let angle: f32 = if diff.length_squared() > 0.5 {
                    diff.y.atan2(diff.x)
                } else {
                    -PI / 2.0
                };

                let rotation = Quat::from_rotation_z(angle);

                let p1 = Vec2::ZERO;
                let p2 = Vec2::new(-size, size * 0.5);
                let p3 = Vec2::new(-size, -size * 0.5);

                let transform_point =
                    |p: Vec2| -> Vec2 { world_position + (rotation * p.extend(0.0)).truncate() };

                let v1 = transform_point(p1);
                let v2 = transform_point(p2);
                let v3 = transform_point(p3);

                let points = [v1, v2, v3, v1];

                gizmos.linestrip_2d(points, color);
            }
        }
    }
}
