use std::collections::HashMap;

use bevy::prelude::*;

use crate::content::{entity::EntityType, ship::base::BoundingBox};

#[derive(Component)]
pub struct Selectable;

#[derive(Resource, Default)]
pub struct SelectableRes {
    state: SelectableState,
}

#[derive(Message)]
pub struct SelectedEvent {
    pub selected: HashMap<EntityType, Vec<Entity>>,
}

#[derive(Default)]
pub enum SelectableState {
    #[default]
    Unselected,
    Selecting {
        start_position: Vec2,
    },
}

pub struct SelectablePlugin;

impl Plugin for SelectablePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectableRes>()
            .add_message::<SelectedEvent>()
            .add_systems(Update, update_selectable);
    }
}

fn update_selectable(
    mut res: ResMut<SelectableRes>,
    mut selected_writer: MessageWriter<SelectedEvent>,
    mut gizmos: Gizmos,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    keyboard: Res<ButtonInput<KeyCode>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    selectable_query: Query<
        (Entity, &EntityType, &GlobalTransform, Option<&BoundingBox>),
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

            let mut selected: HashMap<EntityType, Vec<Entity>> = HashMap::new();

            for (entity, entity_type, transform, bounding_box) in selectable_query.iter() {
                let translation = transform.translation().truncate();

                if let Some(bounding_box) = bounding_box {
                    let edge = bounding_box.size + translation;

                    if top_left.x < edge.x
                        && top_left.y < edge.y
                        && bottom_right.x > edge.x
                        && bottom_right.y > edge.y
                    {
                        selected.entry(*entity_type).or_default().push(entity);
                    }
                } else if top_left.x < translation.x
                    && top_left.y < translation.y
                    && bottom_right.x > translation.x
                    && bottom_right.y > translation.y
                {
                    selected.entry(*entity_type).or_default().push(entity);
                }
            }

            res.state = SelectableState::Unselected;

            if !selected.is_empty() {
                selected_writer.write(SelectedEvent { selected });
            }
        }
    }
}
