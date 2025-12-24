use bevy::prelude::*;

use crate::{resource::SceneState, scene::system::clean_scene_items};
pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            clean_scene_items.run_if(on_message::<StateTransitionEvent<SceneState>>),
        );
    }
}
