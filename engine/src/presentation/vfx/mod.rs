use bevy::prelude::*;
use crate::common::GameState;
use crate::common::SimPostPhysicsSet;

pub mod flash;
pub mod muzzle_flash;

pub use flash::HitFlash;
pub use muzzle_flash::MuzzleFlash;

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
        app.add_systems(
            PostUpdate,
            (
                muzzle_flash::spawn_muzzle_flash,
                muzzle_flash::update_muzzle_flash,
            )
                .chain()
                .after(SimPostPhysicsSet)
                .before(TransformSystems::Propagate)
                .run_if(in_state(GameState::Playing)),
        );
    }
}







