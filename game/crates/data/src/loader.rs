use std::marker::PhantomData;

use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::ecs::component::Mutable;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::de::DeserializeOwned;
use thiserror::Error;

use crate::config::{
    ArenaConfig, EnemiesConfig, ExtractionConfig, PlayerConfig, UpgradesConfig, VisualsConfig,
    WeaponsConfig,
};
use crate::events::ConfigHotReloadedMessage;
use crate::state::{GameState, SimUpdateSet};

#[derive(Debug, Error)]
pub enum RonLoaderError {
    #[error("Failed to read RON asset bytes: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse RON asset: {0}")]
    Ron(#[from] ron::error::SpannedError),
}

#[derive(TypePath)]
pub struct RonAssetLoader<T>(PhantomData<fn() -> T>);

impl<T> Default for RonAssetLoader<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T> AssetLoader for RonAssetLoader<T>
where
    T: Asset + DeserializeOwned + TypePath + Send + Sync + 'static,
{
    type Asset = T;
    type Settings = ();
    type Error = RonLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let parsed = ron::de::from_bytes::<T>(&bytes)?;
        Ok(parsed)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

#[derive(Resource)]
pub struct ConfigHandles {
    pub arena: Handle<ArenaConfig>,
    pub player: Handle<PlayerConfig>,
    pub weapons: Handle<WeaponsConfig>,
    pub enemies: Handle<EnemiesConfig>,
    pub extraction: Handle<ExtractionConfig>,
    pub upgrades: Handle<UpgradesConfig>,
    pub visuals: Handle<VisualsConfig>,
}

pub struct DataPlugin;

impl Plugin for DataPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<ArenaConfig>()
            .init_asset::<PlayerConfig>()
            .init_asset::<WeaponsConfig>()
            .init_asset::<EnemiesConfig>()
            .init_asset::<ExtractionConfig>()
            .init_asset::<UpgradesConfig>()
            .init_asset::<VisualsConfig>()
            .register_asset_loader(RonAssetLoader::<ArenaConfig>::default())
            .register_asset_loader(RonAssetLoader::<PlayerConfig>::default())
            .register_asset_loader(RonAssetLoader::<WeaponsConfig>::default())
            .register_asset_loader(RonAssetLoader::<EnemiesConfig>::default())
            .register_asset_loader(RonAssetLoader::<ExtractionConfig>::default())
            .register_asset_loader(RonAssetLoader::<UpgradesConfig>::default())
            .register_asset_loader(RonAssetLoader::<VisualsConfig>::default())
            .init_resource::<ArenaConfig>()
            .init_resource::<PlayerConfig>()
            .init_resource::<WeaponsConfig>()
            .init_resource::<EnemiesConfig>()
            .init_resource::<ExtractionConfig>()
            .init_resource::<UpgradesConfig>()
            .init_resource::<VisualsConfig>()
            .add_systems(Startup, start_loading_configs)
            .add_systems(
                Update,
                check_initial_configs_loaded.run_if(in_state(GameState::Loading)),
            )
            .add_systems(
                Update,
                (
                    hot_reload_config::<ArenaConfig>,
                    hot_reload_config::<PlayerConfig>,
                    hot_reload_config::<WeaponsConfig>,
                    hot_reload_config::<EnemiesConfig>,
                    hot_reload_config::<ExtractionConfig>,
                    hot_reload_config::<UpgradesConfig>,
                    hot_reload_config::<VisualsConfig>,
                )
                    .before(SimUpdateSet),
            );
    }
}

fn start_loading_configs(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(ConfigHandles {
        arena: asset_server.load("config/arena.ron"),
        player: asset_server.load("config/player.ron"),
        weapons: asset_server.load("config/weapons.ron"),
        enemies: asset_server.load("config/enemies.ron"),
        extraction: asset_server.load("config/extraction.ron"),
        upgrades: asset_server.load("config/upgrades.ron"),
        visuals: asset_server.load("config/visuals.ron"),
    });
}

#[derive(SystemParam)]
struct ConfigAssetStores<'w> {
    arena: Res<'w, Assets<ArenaConfig>>,
    player: Res<'w, Assets<PlayerConfig>>,
    weapons: Res<'w, Assets<WeaponsConfig>>,
    enemies: Res<'w, Assets<EnemiesConfig>>,
    extraction: Res<'w, Assets<ExtractionConfig>>,
    upgrades: Res<'w, Assets<UpgradesConfig>>,
    visuals: Res<'w, Assets<VisualsConfig>>,
}

#[derive(SystemParam)]
struct ActiveConfigs<'w> {
    arena: ResMut<'w, ArenaConfig>,
    player: ResMut<'w, PlayerConfig>,
    weapons: ResMut<'w, WeaponsConfig>,
    enemies: ResMut<'w, EnemiesConfig>,
    extraction: ResMut<'w, ExtractionConfig>,
    upgrades: ResMut<'w, UpgradesConfig>,
    visuals: ResMut<'w, VisualsConfig>,
}

fn check_initial_configs_loaded(
    handles: Res<ConfigHandles>,
    stores: ConfigAssetStores,
    mut active: ActiveConfigs,
    mut reloaded_writer: MessageWriter<ConfigHotReloadedMessage>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Some(arena) = stores.arena.get(&handles.arena) else {
        return;
    };
    let Some(player) = stores.player.get(&handles.player) else {
        return;
    };
    let Some(weapons) = stores.weapons.get(&handles.weapons) else {
        return;
    };
    let Some(enemies) = stores.enemies.get(&handles.enemies) else {
        return;
    };
    let Some(extraction) = stores.extraction.get(&handles.extraction) else {
        return;
    };
    let Some(upgrades) = stores.upgrades.get(&handles.upgrades) else {
        return;
    };
    let Some(visuals) = stores.visuals.get(&handles.visuals) else {
        return;
    };

    *active.arena = arena.clone();
    *active.player = player.clone();
    *active.weapons = weapons.clone();
    *active.enemies = enemies.clone();
    *active.extraction = extraction.clone();
    *active.upgrades = upgrades.clone();
    *active.visuals = visuals.clone();

    reloaded_writer.write(ConfigHotReloadedMessage);
    next_state.set(GameState::Playing);
}

fn hot_reload_config<T: Asset + Resource<Mutability = Mutable> + Clone>(
    mut events: MessageReader<AssetEvent<T>>,
    assets: Res<Assets<T>>,
    mut active: ResMut<T>,
    mut reloaded_writer: MessageWriter<ConfigHotReloadedMessage>,
) {
    let mut updated = false;
    for event in events.read() {
        if let AssetEvent::Modified { id } = event {
            if let Some(asset) = assets.get(*id) {
                *active = asset.clone();
                updated = true;
            }
        }
    }
    if updated {
        reloaded_writer.write(ConfigHotReloadedMessage);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_ron_config_files_parse_cleanly() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..");
        let base = if root.join("core").join("assets").join("config").exists() {
            root.join("core").join("assets").join("config")
        } else {
            root.join("assets").join("config")
        };

        let arena_str = std::fs::read_to_string(base.join("arena.ron")).unwrap();
        let arena: ArenaConfig = ron::de::from_str(&arena_str).unwrap();
        assert!(arena.half_size > 0.0);

        let player_str = std::fs::read_to_string(base.join("player.ron")).unwrap();
        let player: PlayerConfig = ron::de::from_str(&player_str).unwrap();
        assert!(player.max_hp > 0.0);

        let weapons_str = std::fs::read_to_string(base.join("weapons.ron")).unwrap();
        let weapons: WeaponsConfig = ron::de::from_str(&weapons_str).unwrap();
        assert_eq!(weapons.weapons.len(), 2);

        let enemies_str = std::fs::read_to_string(base.join("enemies.ron")).unwrap();
        let enemies: EnemiesConfig = ron::de::from_str(&enemies_str).unwrap();
        assert!(enemies.walker.max_hp > 0.0);

        let extraction_str = std::fs::read_to_string(base.join("extraction.ron")).unwrap();
        let extraction: ExtractionConfig = ron::de::from_str(&extraction_str).unwrap();
        assert!(extraction.hold_duration_secs > 0.0);

        let upgrades_str = std::fs::read_to_string(base.join("upgrades.ron")).unwrap();
        let upgrades: UpgradesConfig = ron::de::from_str(&upgrades_str).unwrap();
        assert!(upgrades.damage_max_level > 0);

        let visuals_str = std::fs::read_to_string(base.join("visuals.ron")).unwrap();
        let visuals: VisualsConfig = ron::de::from_str(&visuals_str).unwrap();
        assert!(!visuals.player.model_path.is_empty());
        assert!(visuals.player.scale > 0.0);
    }
}
