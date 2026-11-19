use super::*;
use crate::content::{ship::BoundingBox, team::Team};
use bevy::prelude::*;


pub(super) fn update_selectable(
    mut res: ResMut<SelectableRes>,
    mut selected_writer: MessageWriter<EntitySelectEvent>,
    mut gizmos: Gizmos,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    keyboard: Res<ButtonInput<KeyCode>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    selectable_query: Query<
        (Entity, &GlobalTransform, Option<&BoundingBox>, &Team, &Visibility),
        With<Selectable>,
    >,
) {
    match &res.state {
        SelectableState::Unselected => {
            let (camera, camera_transform) = *camera_query;

            let Some(cursor_position) = window.cursor_position() else {
                return;
            };

            let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position)
            else {
                return;
            };

            if mouse.just_pressed(MouseButton::Left) && keyboard.pressed(KeyCode::ControlLeft) {
                res.state = SelectableState::Selecting {
                    start_position: world_position,
                };
            }
        }
        SelectableState::Selecting { start_position } => {
            let (camera, camera_transform) = *camera_query;

            let Some(cursor_position) = window.cursor_position() else {
                return;
            };

            let Ok(end_position) = camera.viewport_to_world_2d(camera_transform, cursor_position)
            else {
                return;
            };

            let top_left = Vec2::new(
                start_position.x.min(end_position.x),
                start_position.y.min(end_position.y),
            );

            let bottom_right = Vec2::new(
                start_position.x.max(end_position.x),
                start_position.y.max(end_position.y),
            );

            gizmos.line_2d(
                top_left,
                Vec2::new(bottom_right.x, top_left.y),
                Color::WHITE,
            );
            gizmos.line_2d(
                top_left,
                Vec2::new(top_left.x, bottom_right.y),
                Color::WHITE,
            );
            gizmos.line_2d(
                bottom_right,
                Vec2::new(bottom_right.x, top_left.y),
                Color::WHITE,
            );
            gizmos.line_2d(
                bottom_right,
                Vec2::new(top_left.x, bottom_right.y),
                Color::WHITE,
            );

            if mouse.pressed(MouseButton::Left) && keyboard.pressed(KeyCode::ControlLeft) {
                return;
            }

            let mut selected: Vec<Entity> = Vec::new();

            for (entity, transform, bounding_box, team, visibility) in selectable_query.iter() {
                if !is_selectable(team, visibility) {
                    continue;
                }

                if let Some(bounding_box) = bounding_box {
                    if overlaps_marquee(transform, bounding_box, top_left, bottom_right) {
                        selected.push(entity);
                    }
                    continue;
                }

                let translation = transform.translation().truncate();
                if top_left.x <= translation.x
                    && top_left.y <= translation.y
                    && bottom_right.x >= translation.x
                    && bottom_right.y >= translation.y
                {
                    selected.push(entity);
                }
            }

            res.state = SelectableState::Unselected;

            if !selected.is_empty() {
                selected_writer.write(EntitySelectEvent { selected });
            }
        }
    }
}

fn world_aabb(transform: &GlobalTransform, bounds: &BoundingBox) -> (Vec2, Vec2) {
    let mut min = Vec2::splat(f32::INFINITY);
    let mut max = Vec2::splat(f32::NEG_INFINITY);
    for corner in bounds.corners() {
        let world = transform.transform_point(corner.extend(0.0)).truncate();
        min = min.min(world);
        max = max.max(world);
    }
    (min, max)
}

fn overlaps_marquee(
    transform: &GlobalTransform,
    bounds: &BoundingBox,
    rect_min: Vec2,
    rect_max: Vec2,
) -> bool {
    let (min, max) = world_aabb(transform, bounds);
    rect_min.x <= max.x && rect_min.y <= max.y && rect_max.x >= min.x && rect_max.y >= min.y
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marquee_overlap_uses_offset_bounds() {
        let bounds = BoundingBox::from_min_max(Vec2::ZERO, Vec2::new(64.0, 32.0));
        let transform = GlobalTransform::from_xyz(100.0, 200.0, 0.0);

        // Inside [100..164, 200..232]
        assert!(overlaps_marquee(
            &transform,
            &bounds,
            Vec2::new(150.0, 210.0),
            Vec2::new(170.0, 220.0),
        ));

        // Left of origin [68..99, 200..232] should NOT match (old centered box would have!)
        assert!(!overlaps_marquee(
            &transform,
            &bounds,
            Vec2::new(70.0, 205.0),
            Vec2::new(95.0, 225.0),
        ));
    }

    #[test]
    fn marquee_overlap_accounts_for_rotation() {
        let bounds = BoundingBox::from_min_max(Vec2::ZERO, Vec2::new(100.0, 20.0));
        let mut local = Transform::from_xyz(0.0, 0.0, 0.0);
        local.rotate_z(std::f32::consts::FRAC_PI_2);
        let transform = GlobalTransform::from(local);

        // Rotated +90 deg: [0..100, 0..20] becomes [-20..0, 0..100]
        assert!(overlaps_marquee(
            &transform,
            &bounds,
            Vec2::new(-15.0, 80.0),
            Vec2::new(-5.0, 95.0),
        ));

        // Along +X where unrotated ship would be, rotated ship is no longer there
        assert!(!overlaps_marquee(
            &transform,
            &bounds,
            Vec2::new(50.0, 5.0),
            Vec2::new(90.0, 15.0),
        ));
    }
}
