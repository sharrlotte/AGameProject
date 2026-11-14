use bevy::prelude::*;
use crate::config::AppRonConfigExt;
use crate::config::HazardConfig;
use crate::common::{GameScene, GameState, SimUpdateSet};

pub mod state;
pub mod system;

pub use state::{HazardMarker, HazardSpec, HazardSpawnMessage, HazardZone};
pub use system::{spawn_hazards, tick_hazards};

pub struct HazardPlugin;

impl Plugin for HazardPlugin {
    fn build(&self, app: &mut App) {
        app.register_ron_config::<HazardConfig>()
            .add_message::<HazardSpawnMessage>()
            .add_systems(
                Update,
                (system::tick_hazards, system::spawn_hazards)
                    .chain()
                    .in_set(SimUpdateSet)
                    .run_if(in_state(GameState::Playing))
                    .run_if(in_state(GameScene::Arena)),
            );
    }
}

#[cfg(test)]
mod tests;
