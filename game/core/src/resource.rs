use bevy::{
    ecs::{component::Component, entity::Entity, resource::Resource},
    state::state::States,
};

use crate::save::io::SchematicData;

#[derive(Clone, Copy, Eq, Hash, States, PartialEq, Debug, Default)]
pub enum SceneState {
    #[default]
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

#[derive(Default, PartialEq, Clone, Debug)]
pub enum TurretControl {
    #[default]
    Auto,
    Manual,
    Hold,
}

impl TurretControl {
    pub fn next(&mut self) -> Self {
        match self {
            Self::Auto => Self::Manual,
            Self::Manual => Self::Hold,
            Self::Hold => Self::Auto,
        }
    }
}

#[derive(Resource, Default, Debug)]
pub struct GameVars {
    pub current_ship: Option<SchematicData>,
    pub turret_control: TurretControl,
    pub controlling: Option<Entity>,
    pub following: Option<Entity>,
}
