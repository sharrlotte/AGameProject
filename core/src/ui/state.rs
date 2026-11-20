use bevy::{ecs::resource::Resource, prelude::Reflect, state::state::States};

#[derive(Clone, Copy, Eq, Hash, States, PartialEq, Debug, Default, Reflect)]
pub enum SceneState {
    #[default]
    Loading,
    Home,
    Game,
    ShipEditor,
}

#[derive(Clone, Copy, Eq, Hash, States, PartialEq, Debug, Default)]
pub enum PauseState {
    #[default]
    Playing,
    Paused,
}

#[derive(Clone, Copy, Eq, Hash, States, PartialEq, Debug, Default)]
pub enum SettingState {
    #[default]
    Closed,
    Open,
}

#[derive(Clone, Copy, Eq, Hash, States, PartialEq, Debug, Default)]
pub enum SlotPickerState {
    #[default]
    Closed,
    Open,
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
