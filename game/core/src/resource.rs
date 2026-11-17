use bevy::{
    ecs::{component::Component, entity::Entity, resource::Resource},
    state::state::States,
};

use crate::save::io::SchematicData;

#[derive(Clone, Copy, Eq, Hash, States, PartialEq, Debug, Default)]
pub enum SceneState {
    #[default]
    Loading,
    Home,
    Game,
    ShipEditor,
}

#[derive(Component, Clone, Copy, Eq, Hash, States, PartialEq, Debug, Default)]
pub struct SceneEntity(pub SceneState);

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

#[derive(Resource, Default, Debug)]
pub struct DeployPayload {
    pub current_ship: Option<SchematicData>,
}

#[derive(Resource, Default, Debug)]
pub struct CameraTarget {
    pub following: Option<Entity>,
}

#[derive(Resource, Default, Debug)]
pub struct ControlledShip {
    pub controlling: Option<Entity>,
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
