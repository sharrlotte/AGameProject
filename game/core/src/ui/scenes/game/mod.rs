mod background;
mod indicators;
mod ship;

use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

use crate::content::structures::block::BlockGroup;
use crate::content::structures::ship::PlayerShip;
use crate::ui::scenes::game::background::BackgroundPlugin;
use crate::ui::scenes::game::indicators::IndicatorsPlugin;
use crate::{AppState, ui::scenes::game::ship::ShipPlugin};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Game), setup_game)
            .add_systems(OnExit(AppState::Game), cleanup_game)
            .add_systems(
                Update,
                (camera_follow, camera_zoom).run_if(in_state(AppState::Game)),
            )
            .add_plugins(ShipPlugin)
            .add_plugins(BackgroundPlugin)
            .add_plugins(IndicatorsPlugin);
    }
}

fn setup_game(_commands: Commands) {
    info!("Setting up Game Scene");
}

fn cleanup_game(_commands: Commands) {
    info!("Cleaning up Game Scene");
}

fn camera_follow(
    player: Single<(&Transform, Option<&Children>), (With<PlayerShip>, Without<Camera2d>)>,
    block_query: Query<(&GlobalTransform, &BlockGroup)>,
    mut camera_transform: Single<&mut Transform, (With<Camera2d>, Without<PlayerShip>)>,
    time: Res<Time>,
) {
    let (ship_transform, children) = *player;

    let mut target = ship_transform.translation.truncate();

    if let Some(children) = children {
        for child in children.iter() {
            if let Ok((global_transform, group)) = block_query.get(child) {
                if *group == BlockGroup::Core {
                    target = global_transform.translation().truncate();
                    break;
                }
            }
        }
    }

    let current = camera_transform.translation.truncate();

    // Smooth follow
    let smooth_speed = 5.0;
    let new_pos = current.lerp(target, time.delta_secs() * smooth_speed);

    camera_transform.translation.x = new_pos.x;
    camera_transform.translation.y = new_pos.y;
}

fn camera_zoom(
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    mut camera_query: Query<&mut Projection, With<Camera2d>>,
) {
    for event in mouse_wheel_events.read() {
        for mut projection in camera_query.iter_mut() {
            if let Projection::Orthographic(ortho) = &mut *projection {
                let zoom_speed = 0.1;
                ortho.scale -= event.y * zoom_speed * ortho.scale;
                ortho.scale = ortho.scale.clamp(0.1, 5.0);
            }
        }
    }
}
