use bevy::prelude::*;
use data::{GameState, SimPostPhysicsSet};

pub mod flash;

pub use flash::HitFlash;

pub struct VfxPlugin;

impl Plugin for VfxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            flash::update_hit_flash
                .after(SimPostPhysicsSet)
                .before(TransformSystems::Propagate)
                .run_if(in_state(GameState::Playing)),
        );
    }
}
