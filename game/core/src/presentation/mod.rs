use bevy::prelude::*;
use bevy_rapier3d::prelude::PhysicsSet;

use crate::GameState;
use crate::sim::{SimPostPhysicsSet, SimUpdateSet};

pub mod camera;
pub mod input;
pub mod ui;
pub mod visuals;

pub struct PresentationPlugin;

impl Plugin for PresentationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<visuals::SharedVisualMaterials>()
            .add_systems(Startup, (camera::spawn_camera, visuals::setup_lighting))
            .add_systems(OnEnter(GameState::Playing), ui::spawn_run_hud)
            .add_systems(OnExit(GameState::Playing), ui::despawn_run_hud)
            .add_systems(OnEnter(GameState::GameOver), ui::spawn_game_over_ui)
            .add_systems(OnExit(GameState::GameOver), ui::despawn_game_over_ui)
            .add_systems(
                Update,
                (
                    visuals::attach_arena_visuals,
                    visuals::attach_actor_visuals,
                ),
            )
            .add_systems(
                Update,
                (
                    input::gather_local_input.before(SimUpdateSet),
                    ui::update_run_hud.after(SimUpdateSet),
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                PostUpdate,
                (
                    camera::follow_local_player.after(PhysicsSet::Writeback),
                    visuals::sync_visual_facing.after(SimPostPhysicsSet),
                    visuals::update_hit_flash.after(SimPostPhysicsSet),
                )
                    .before(TransformSystems::Propagate)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                ui::handle_restart_shortcut.run_if(in_state(GameState::GameOver)),
            );
    }
}
