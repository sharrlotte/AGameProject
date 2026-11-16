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
pub struct GameVars {
    pub current_ship: Option<SchematicData>,
    pub controlling: Option<Entity>,
    pub following: Option<Entity>,
}
