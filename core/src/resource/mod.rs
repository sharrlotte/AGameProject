pub use crate::control::*;
pub use crate::ui::*;

use bevy::ecs::resource::Resource;

use crate::content::ship::ShipDesign;

#[derive(Resource, Default, Debug)]
pub struct DeployPayload {
    pub fleet: Vec<ShipDesign>,
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
pub enum BattleMode {
    #[default]
    Campaign,
    QuickBattle,
}


