use bevy::prelude::*;

#[derive(Component, Default, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct FireIntent {
    pub active: bool,
    pub target_pos: Vec2,
}

#[derive(Component, Default, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct PlayerFireController;

#[derive(Component, Default, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct AIFireController;

pub fn update_player_fire_controller(
    mut query: Query<(&mut FireIntent, &GlobalTransform), With<PlayerFireController>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
    mouse_input: Res<ButtonInput<MouseButton>>,
) {
    let (camera, camera_transform) = *camera_query;

    if let Some(cursor_position) = window.cursor_position() {
        if let Ok(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position) {
            for (mut intent, _) in query.iter_mut() {
                intent.target_pos = world_position;
                intent.active = mouse_input.pressed(MouseButton::Left);
            }
        }
    }
}
