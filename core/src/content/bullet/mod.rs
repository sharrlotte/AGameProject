pub mod component;
pub mod system;

pub use component::*;
pub use system::*;

use bevy::prelude::*;

use crate::content::block::feature::turret::advance_turret_charge;
use crate::resource::SceneState;
use crate::simulation::SimulationSet;

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                handle_missiles,
                handle_single_shot.after(advance_turret_charge),
                handle_burst_shot.after(advance_turret_charge),
                handle_missile_radar_target,
                handle_laser_beam,
                handle_pulling_beam,
                destroy_out_of_range_bullet,
                handle_collisions,
            )
                .run_if(in_state(SceneState::Game))
                .in_set(SimulationSet::Weapons),
        )
        .add_systems(
            FixedUpdate,
            handle_point_defense_hits
                .run_if(in_state(SceneState::Game))
                .in_set(SimulationSet::Weapons),
        )
        .add_systems(
            FixedPostUpdate,
            (handle_laser)
                .run_if(in_state(SceneState::Game))
                .in_set(SimulationSet::Damage),
        );
    }
}
