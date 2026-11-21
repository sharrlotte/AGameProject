pub use crate::control::*;
pub use crate::ui::*;

pub mod session;

pub use session::SessionClock;

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

    pub fn next_u64(&mut self) -> u64 {
        let mut value = self.seed;
        value ^= value >> 12;
        value ^= value << 25;
        value ^= value >> 27;
        self.seed = value;
        value.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
}

impl Default for GameRng {
    fn default() -> Self {
        Self::new(0x5EED_1234_5678_9ABC)
    }
}


