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
    mouse_input: Res<ButtonInput<MouseButton>>,
    selectable_query: Query<
        (Entity, &EntityType, &GlobalTransform, Option<&BoundingBox>),
        With<Selectable>,
    >,
    window: Single<&Window>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    mut selected_writer: MessageWriter<SelectedEvent>,
) {
    match &res.state {
        SelectableState::Unselected => {
            if !mouse_input.just_pressed(MouseButton::Left) {
                return;
            }

            let (camera, camera_transform) = *camera_query;

            let Some(cursor_position) = window.cursor_position() else {
                return;
            };

            let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position)
            else {
                return;
            };

            res.state = SelectableState::Selecting {
                start_position: world_position,
            };
        }
        SelectableState::Selecting { start_position } => {
            if !mouse_input.just_released(MouseButton::Left) {
                return;
            }

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

            selected_writer.write(SelectedEvent { selected });
        }
    }
}
