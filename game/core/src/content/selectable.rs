use std::collections::HashMap;

use bevy::prelude::*;

use crate::{content::ship::base::BoundingBox, resource::SceneState};

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub enum SelectableType {
    Ship,
}

#[derive(Component)]
pub struct Selectable(pub SelectableType);

#[derive(Component)]
pub struct SelectableUI;

#[derive(Resource, Default)]
pub struct SelectableRes {
    state: SelectableState,
}

#[derive(Message)]
pub struct SelectedEvent {
    selected: HashMap<SelectableType, Vec<Entity>>,
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
            .add_systems(OnEnter(SceneState::Game), setup)
            .add_systems(Update, update_selectable);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        SelectableUI,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
    ));
}

fn update_selectable(
    mut res: ResMut<SelectableRes>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    selectable_query: Query<(Entity, &Selectable, &GlobalTransform, Option<&BoundingBox>)>,
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

            let mut selected: HashMap<SelectableType, Vec<Entity>> = HashMap::new();

            for (entity, selectable, transform, bounding_box) in selectable_query.iter() {
                let translation = transform.translation().truncate();

                if let Some(bounding_box) = bounding_box {
                    let edge = bounding_box.size + translation;

                    if top_left.x < edge.x
                        && top_left.y < edge.y
                        && bottom_right.x > edge.x
                        && bottom_right.y > edge.y
                    {
                        selected.entry(selectable.0).or_default().push(entity);
                    }
                } else if top_left.x < translation.x
                    && top_left.y < translation.y
                    && bottom_right.x > translation.x
                    && bottom_right.y > translation.y
                {
                    selected.entry(selectable.0).or_default().push(entity);
                }
            }

            res.state = SelectableState::Unselected;

            selected_writer.write(SelectedEvent { selected });
        }
    }
}
