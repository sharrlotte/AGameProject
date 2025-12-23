use bevy::{ecs::resource::Resource, state::state::States};

use crate::save::component::SchematicData;



#[derive(Clone, Copy, Eq, Hash, States, PartialEq, Debug, Default)]
pub enum SceneState {
    #[default]
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
pub struct GameState {
    pub current_ship: Option<SchematicData>,
    pub turret_control: TurretControl,
}
