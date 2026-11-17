use super::*;
use crate::content::{ship::BoundingBox, team::Team};
use crate::simulation::SimulationSet;
use bevy::prelude::*;

pub struct SelectablePlugin;

impl Plugin for SelectablePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectableRes>()
            .add_message::<EntitySelectEvent>()
            .add_systems(Update, update_selectable.in_set(SimulationSet::Input));
    }
}

fn update_selectable(
    mut res: ResMut<SelectableRes>,
    mut selected_writer: MessageWriter<EntitySelectEvent>,
    mut gizmos: Gizmos,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    keyboard: Res<ButtonInput<KeyCode>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    selectable_query: Query<
        (Entity, &GlobalTransform, Option<&BoundingBox>, &Team),
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

            for (entity, transform, bounding_box, team) in selectable_query.iter() {
                if !team.is_player() {
                    continue;
                }

                let translation = transform.translation().truncate();

                if let Some(bounding_box) = bounding_box {
                    let half = bounding_box.size / 2.0;
                    let min = translation - half;
                    let max = translation + half;

                    if top_left.x <= max.x
                        && top_left.y <= max.y
                        && bottom_right.x >= min.x
                        && bottom_right.y >= min.y
                    {
                        selected.push(entity);
                    }

                    continue;
                }

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
