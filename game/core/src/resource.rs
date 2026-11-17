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

/// Seed persisted in save slots. Kept as a resource so the snapshot can carry
/// it without tying the save schema to a concrete RNG implementation.
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

/// How WASD input is interpreted while in the Game scene.
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlMode {
    /// WASD/QE fly the controlled ship and the camera follows it.
    Pilot,
    /// WASD pan the camera and ship thrust is paused.
    #[default]
    FreeLook,
}

/// A world load that has been requested but not yet applied.
///
/// Set by `handle_load_request`, then consumed when `SceneState::Game` is
/// entered so the old scene is torn down by `DespawnOnExit` before the snapshot
/// is spawned.
#[derive(Resource, Default, Debug)]
pub struct PendingLoad {
    pub slot: Option<u8>,
}

impl PendingLoad {
    pub fn is_pending(&self) -> bool {
        self.slot.is_some()
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
