// TODO: Split into control, ui folder

use bevy::{
    ecs::{entity::Entity, resource::Resource},
    prelude::Reflect,
    state::state::States,
};

use crate::content::ship::ShipDesign;

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

#[derive(Resource, Default, Debug)]
pub struct DeployPayload {
    pub current_ship: Option<ShipDesign>,
}

#[derive(Resource)]
pub struct GameRng {
    seed: u64,
}

impl GameRng {
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }
}

impl Default for GameRng {
    fn default() -> Self {
        Self::new(0x5EED_1234_5678_9ABC)
    }
}

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlMode {
    Pilot,
    #[default]
    FreeLook,
}

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleMode {
    #[default]
    Campaign,
    QuickBattle,
}

#[derive(Resource, Default, Debug)]
pub struct PendingLoad {
    pub active: bool,
}

impl PendingLoad {
    pub fn is_pending(&self) -> bool {
        self.active
    }
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
