pub mod ai;
pub mod block;
pub mod game;
pub(crate) mod loader;
pub mod minimap;
pub mod setting;
pub mod skirmish;

pub use ai::AiConfig;
pub use block::BlocksConfig;
pub use game::SimConfig;
pub use minimap::MinimapConfig;
pub use setting::GameSettings;
pub use skirmish::{ShipSpawn, SkirmishDef};

use std::collections::HashMap;

use bevy::asset::{AssetEvent, LoadState};
use bevy::prelude::*;

use crate::content::block::BlockRegistry;
use crate::quest::component::{QuestCatalogDef, QuestDef, QuestGenConfig, QuestId};
use crate::resource::SceneState;
use crate::simulation::SimulationSet;

#[derive(Clone, Debug, Default)]
pub struct QuestConfigData {
    pub authored: HashMap<QuestId, QuestDef>,
    pub generation: QuestGenConfig,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct GameConfig {
    pub sim: SimConfig,
    pub ai: AiConfig,
    pub minimap: MinimapConfig,
    pub skirmish: SkirmishDef,
    pub quest: QuestConfigData,
}

#[derive(States, Clone, Eq, Hash, PartialEq, Default, Debug)]
pub enum ConfigState {
    #[default]
    Loading,
    Ready,
    Error(ConfigError),
}

#[derive(Clone, Eq, Hash, PartialEq, Debug)]
pub struct ConfigError {
    pub failed: ConfigFile,
    detail: Option<String>,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug)]
pub enum ConfigFile {
    Game,
    Blocks,
    Ai,
    Minimap,
    Skirmish,
    Quests,
    QuestGen,
}

impl ConfigError {
    pub fn with_detail(failed: ConfigFile, detail: impl Into<String>) -> Self {
        Self {
            failed,
            detail: Some(detail.into()),
        }
    }

    pub fn message(&self) -> String {
        let base = match self.failed {
            ConfigFile::Game => "config/base/game.ron failed to load",
            ConfigFile::Blocks => "config/base/block.ron failed to load",
            ConfigFile::Ai => "config/base/ai.ron failed to load",
            ConfigFile::Minimap => "config/base/minimap.ron failed to load",
            ConfigFile::Skirmish => "config/base/skirmish.ron failed to load",
            ConfigFile::Quests => "config/quest/quests.ron failed to load",
            ConfigFile::QuestGen => "config/quest/generation.ron failed to load",
        };

        match &self.detail {
            Some(detail) => format!("{base}: {detail}"),
            None => base.to_string(),
        }
    }
}

#[derive(Resource)]
struct ConfigHandles {
    sim: Handle<SimConfig>,
    blocks: Handle<BlocksConfig>,
    ai: Handle<AiConfig>,
    minimap: Handle<MinimapConfig>,
    skirmish: Handle<SkirmishDef>,
    quests: Handle<QuestCatalogDef>,
    quest_gen: Handle<QuestGenConfig>,
}

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        use crate::config::setting::SettingSaveTimer;

        app.init_state::<ConfigState>()
            .init_resource::<SettingSaveTimer>()
            .init_resource::<GameConfig>()
            .init_resource::<GameSettings>()
            .init_asset::<SimConfig>()
            .init_asset::<BlocksConfig>()
            .init_asset::<AiConfig>()
            .init_asset::<MinimapConfig>()
            .init_asset::<SkirmishDef>()
            .init_asset::<QuestCatalogDef>()
            .init_asset::<QuestGenConfig>()
            .register_asset_loader(loader::RonAssetLoader::<SimConfig>::new(&["ron"]))
            .register_asset_loader(loader::RonAssetLoader::<BlocksConfig>::new(&["ron"]))
            .register_asset_loader(loader::RonAssetLoader::<AiConfig>::new(&["ron"]))
            .register_asset_loader(loader::RonAssetLoader::<MinimapConfig>::new(&["ron"]))
            .register_asset_loader(loader::RonAssetLoader::<SkirmishDef>::new(&["ron"]))
            .register_asset_loader(loader::RonAssetLoader::<QuestCatalogDef>::new(&["ron"]))
            .register_asset_loader(loader::RonAssetLoader::<QuestGenConfig>::new(&["ron"]))
            .add_systems(PreStartup, setting::load_settings)
            .add_systems(Startup, load_config_handles)
            .add_systems(
                Update,
                check_config_ready
                    .run_if(in_state(ConfigState::Loading))
                    .in_set(SimulationSet::Present),
            )
            .add_systems(
                Update,
                (
                    sim_config_bridge,
                    block::blocks_bridge,
                    ai_bridge,
                    minimap_bridge,
                    skirmish_bridge,
                    quests_bridge,
                    quest_gen_bridge,
                    setting::save_setting_on_change,
                )
                    .run_if(in_state(ConfigState::Ready))
                    .in_set(SimulationSet::Present),
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

#[derive(Resource)]
struct ConfigErrorScreen;

#[derive(Component)]
struct ErrorText;

fn load_config_handles(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(ConfigHandles {
        sim: assets.load("config/base/game.ron"),
        blocks: assets.load("config/base/block.ron"),
        ai: assets.load("config/base/ai.ron"),
        minimap: assets.load("config/base/minimap.ron"),
        skirmish: assets.load("config/base/skirmish.ron"),
        quests: assets.load("config/quest/quests.ron"),
        quest_gen: assets.load("config/quest/generation.ron"),
    });
}

fn check_load_failed<A: Asset>(
    assets: &AssetServer,
    handle: &Handle<A>,
    file: ConfigFile,
    config_state: &mut NextState<ConfigState>,
) -> bool {
    if let Some(LoadState::Failed(err)) = assets.get_load_state(handle) {
        config_state.set(ConfigState::Error(ConfigError::with_detail(
            file,
            err.to_string(),
        )));
        return true;
    }
    false
}

fn check_config_ready(
    mut commands: Commands,
    handles: Option<Res<ConfigHandles>>,
    sim_assets: Res<Assets<SimConfig>>,
    block_assets: Res<Assets<BlocksConfig>>,
    ai_assets: Res<Assets<AiConfig>>,
    minimap_assets: Res<Assets<MinimapConfig>>,
    skirmish_assets: Res<Assets<SkirmishDef>>,
    quest_assets: Res<Assets<QuestCatalogDef>>,
    quest_gen_assets: Res<Assets<QuestGenConfig>>,
    assets: Res<AssetServer>,
    mut config_state: ResMut<NextState<ConfigState>>,
    mut scene_state: ResMut<NextState<SceneState>>,
) {
    let Some(handles) = handles else {
        return;
    };

    if check_load_failed(&assets, &handles.sim, ConfigFile::Game, &mut config_state)
        || check_load_failed(&assets, &handles.blocks, ConfigFile::Blocks, &mut config_state)
        || check_load_failed(&assets, &handles.ai, ConfigFile::Ai, &mut config_state)
        || check_load_failed(&assets, &handles.minimap, ConfigFile::Minimap, &mut config_state)
        || check_load_failed(&assets, &handles.skirmish, ConfigFile::Skirmish, &mut config_state)
        || check_load_failed(&assets, &handles.quests, ConfigFile::Quests, &mut config_state)
        || check_load_failed(&assets, &handles.quest_gen, ConfigFile::QuestGen, &mut config_state)
    {
        return;
    }

    let (
        Some(sim),
        Some(blocks),
        Some(ai),
        Some(minimap),
        Some(skirmish),
        Some(quests),
        Some(quest_gen),
    ) = (
        sim_assets.get(&handles.sim),
        block_assets.get(&handles.blocks),
        ai_assets.get(&handles.ai),
        minimap_assets.get(&handles.minimap),
        skirmish_assets.get(&handles.skirmish),
        quest_assets.get(&handles.quests),
        quest_gen_assets.get(&handles.quest_gen),
    )
    else {
        return;
    };

    let mut registry = BlockRegistry::default();
    for (key, block) in &blocks.blocks {
        match block.clone().into_def(assets.as_ref(), key.clone()) {
            Ok(def) => registry.register(key, def),
            Err(error) => {
                config_state.set(ConfigState::Error(ConfigError::with_detail(
                    ConfigFile::Blocks,
                    error.message(),
                )));
                return;
            }
        }
    }

    let authored = quests
        .quests
        .iter()
        .map(|def| (def.id.clone(), def.clone()))
        .collect();

    commands.insert_resource(GameConfig {
        sim: sim.clone(),
        ai: ai.clone(),
        minimap: minimap.clone(),
        skirmish: skirmish.clone(),
        quest: QuestConfigData {
            authored,
            generation: quest_gen.clone(),
        },
    });
    commands.insert_resource(registry);

    config_state.set(ConfigState::Ready);
    scene_state.set(SceneState::Home);
}

fn apply_bridge<A: Asset>(
    events: &mut MessageReader<AssetEvent<A>>,
    assets: &Assets<A>,
    mut apply: impl FnMut(&A),
) {
    for event in events.read() {
        let (AssetEvent::Modified { id } | AssetEvent::LoadedWithDependencies { id }) = event
        else {
            continue;
        };
        if let Some(asset) = assets.get(*id) {
            apply(asset);
        }
    }
}

fn sim_config_bridge(
    mut events: MessageReader<AssetEvent<SimConfig>>,
    assets: Res<Assets<SimConfig>>,
    mut config: ResMut<GameConfig>,
) {
    apply_bridge(&mut events, &assets, |v| config.sim = v.clone());
}

fn ai_bridge(
    mut events: MessageReader<AssetEvent<AiConfig>>,
    assets: Res<Assets<AiConfig>>,
    mut config: ResMut<GameConfig>,
) {
    apply_bridge(&mut events, &assets, |v| config.ai = v.clone());
}

fn minimap_bridge(
    mut events: MessageReader<AssetEvent<MinimapConfig>>,
    assets: Res<Assets<MinimapConfig>>,
    mut config: ResMut<GameConfig>,
) {
    apply_bridge(&mut events, &assets, |v| config.minimap = v.clone());
}

fn skirmish_bridge(
    mut events: MessageReader<AssetEvent<SkirmishDef>>,
    assets: Res<Assets<SkirmishDef>>,
    mut config: ResMut<GameConfig>,
) {
    apply_bridge(&mut events, &assets, |v| config.skirmish = v.clone());
}

fn quests_bridge(
    mut events: MessageReader<AssetEvent<QuestCatalogDef>>,
    assets: Res<Assets<QuestCatalogDef>>,
    mut config: ResMut<GameConfig>,
) {
    apply_bridge(&mut events, &assets, |catalog| {
        config.quest.authored = catalog
            .quests
            .iter()
            .map(|def| (def.id.clone(), def.clone()))
            .collect();
    });
}

fn quest_gen_bridge(
    mut events: MessageReader<AssetEvent<QuestGenConfig>>,
    assets: Res<Assets<QuestGenConfig>>,
    mut config: ResMut<GameConfig>,
) {
    apply_bridge(&mut events, &assets, |v| config.quest.generation = v.clone());
}

fn config_failed(state: Res<State<ConfigState>>) -> bool {
    matches!(state.get(), ConfigState::Error(_))
}

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
                ErrorText,
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
    mut text: Query<&mut Text, With<ErrorText>>,
) {
    let ConfigState::Error(error) = state.get() else {
        return;
    };
    for mut text in &mut text {
        **text = error.message().to_string();
    }
}
