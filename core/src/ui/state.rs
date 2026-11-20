use bevy::{ecs::resource::Resource, prelude::Reflect, state::state::States};

#[derive(Clone, Copy, Eq, Hash, States, PartialEq, Debug, Default, Reflect)]
pub enum SceneState {
    #[default]
    Loading,
    Home,
    Game,
    ShipEditor,
}

#[derive(Resource, Default, Debug)]
pub struct AppNotice {
    pub message: String,
    pub remaining: f32,
}

impl AppNotice {
    pub fn show(&mut self, message: impl Into<String>) {
        self.message = message.into();
        self.remaining = 5.0;
    }
}
