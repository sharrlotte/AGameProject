mod block;
mod game;
pub(crate) mod loader;
mod setting;

pub use block::BlocksConfig;
pub use game::{GameConfig, GameConfigValue};
pub use setting::GameSetting;

use bevy::asset::LoadState;
use bevy::prelude::*;

use crate::content::block::BlockRegistry;
use crate::resource::SceneState;
use crate::simulation::SimulationSet;

#[derive(States, Clone, Copy, Eq, Hash, PartialEq, Default, Debug)]
pub enum ConfigState {
    #[default]
    Loading,
    Ready,
}

#[derive(Resource)]
struct ConfigHandles {
    game: Handle<GameConfig>,
    blocks: Handle<BlocksConfig>,
    settings: Handle<GameSetting>,
}

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ConfigState>()
            .init_asset::<GameConfig>()
            .init_asset::<BlocksConfig>()
            .init_asset::<GameSetting>()
            .register_asset_loader(loader::RonAssetLoader::<GameConfig>::new(&["ron"]))
            .register_asset_loader(loader::RonAssetLoader::<BlocksConfig>::new(&["ron"]))
            .register_asset_loader(loader::RonAssetLoader::<GameSetting>::new(&["ron"]))
            .add_systems(Startup, load_config_handles)
            .add_systems(
                Update,
                check_config_ready
                    .run_if(in_state(ConfigState::Loading))
                    .in_set(SimulationSet::Apply),
            )
            .add_systems(
                Update,
                (
                    game::game_config_bridge,
                    block::blocks_bridge,
                    setting::game_setting_bridge,
                    setting::save_setting_on_change,
                )
                    .run_if(in_state(ConfigState::Ready))
                    .in_set(SimulationSet::Apply),
            );

    }
}

fn load_config_handles(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(ConfigHandles {
        game: assets.load("config/base/game.ron"),
        blocks: assets.load("config/base/block.ron"),
        settings: assets.load("config/setting.ron"),
    });
}

fn check_config_ready(
    mut commands: Commands,
    handles: Option<Res<ConfigHandles>>,
    game_assets: Res<Assets<GameConfig>>,
    block_assets: Res<Assets<BlocksConfig>>,
    setting_assets: Res<Assets<GameSetting>>,
    assets: Res<AssetServer>,
    mut config_state: ResMut<NextState<ConfigState>>,
    mut scene_state: ResMut<NextState<SceneState>>,
) {
    let Some(handles) = handles else {
        return;
    };

    if matches!(
        assets.get_load_state(&handles.game),
        Some(LoadState::Failed(_))
    ) {
        panic!("config/base/game.ron failed to load");
    }

    if matches!(
        assets.get_load_state(&handles.blocks),
        Some(LoadState::Failed(_))
    ) {
        panic!("config/base/block.ron failed to load");
    }

    let Some(game) = game_assets.get(&handles.game) else {
        return;
    };

    let Some(blocks) = block_assets.get(&handles.blocks) else {
        return;
    };

    let mut registry = BlockRegistry::default();

    for block in &blocks.blocks {
        registry.register(block.clone().into_def(assets.as_ref()));
    }

    if matches!(
        assets.get_load_state(&handles.settings),
        Some(LoadState::Failed(_))
    ) {
        warn!("config/setting.ron failed to load; using defaults");
    }

    commands.insert_resource(GameConfigValue(game.clone()));
    commands.insert_resource(registry);
    commands.insert_resource(
        setting_assets
            .get(&handles.settings)
            .cloned()
            .unwrap_or_default(),
    );

    config_state.set(ConfigState::Ready);
    scene_state.set(SceneState::Home);
}
