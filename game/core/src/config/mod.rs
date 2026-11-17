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
    Error(ConfigError),
}

#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug)]
pub struct ConfigError {
    pub failed: ConfigFile,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug)]
pub enum ConfigFile {
    Game,
    Blocks,
    Settings,
}

impl ConfigError {
    fn new(failed: ConfigFile) -> Self {
        Self { failed }
    }

    pub fn message(&self) -> &'static str {
        match self.failed {
            ConfigFile::Game => "config/base/game.ron failed to load",
            ConfigFile::Blocks => "config/base/block.ron failed to load",
            ConfigFile::Settings => "config/setting.ron failed to load",
        }
    }
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
            )
            .add_systems(
                Update,
                (
                    spawn_config_error_screen.run_if(config_failed),
                    refresh_config_error_text.run_if(resource_exists::<ConfigErrorScreen>),
                ),
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
        config_state.set(ConfigState::Error(ConfigError::new(ConfigFile::Game)));
        return;
    }

    if matches!(
        assets.get_load_state(&handles.blocks),
        Some(LoadState::Failed(_))
    ) {
        config_state.set(ConfigState::Error(ConfigError::new(ConfigFile::Blocks)));
        return;
    }

    let Some(game) = game_assets.get(&handles.game) else {
        return;
    };

    let Some(blocks) = block_assets.get(&handles.blocks) else {
        return;
    };

    let mut registry = BlockRegistry::default();

    for (key, block) in &blocks.blocks {
        registry.register(key, block.clone().into_def(assets.as_ref(), key.clone()));
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

fn config_failed(state: Res<State<ConfigState>>) -> bool {
    matches!(state.get(), ConfigState::Error(_))
}

#[derive(Resource)]
struct ConfigErrorScreen;

#[derive(Component)]
struct ConfigErrorText;

fn spawn_config_error_screen(
    mut commands: Commands,
    state: Res<State<ConfigState>>,
    existing: Option<Res<ConfigErrorScreen>>,
) {
    if existing.is_some() {
        return;
    }

    let ConfigState::Error(error) = state.get() else {
        return;
    };

    commands.insert_resource(ConfigErrorScreen);
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.08, 0.02, 0.02)),
        ))
        .with_children(|root| {
            root.spawn((
                Text::new("CONFIGURATION ERROR"),
                TextFont {
                    font_size: FontSize::Px(36.0),
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.35, 0.35)),
            ));
            root.spawn((
                Text::new(error.message()),
                TextFont {
                    font_size: FontSize::Px(18.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                ConfigErrorText,
            ));
            root.spawn((
                Text::new("Fix the file and restart the game."),
                TextFont {
                    font_size: FontSize::Px(14.0),
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
}

fn refresh_config_error_text(
    state: Res<State<ConfigState>>,
    mut text: Query<&mut Text, With<ConfigErrorText>>,
) {
    let ConfigState::Error(error) = state.get() else {
        return;
    };
    for mut text in &mut text {
        **text = error.message().to_string();
    }
}
