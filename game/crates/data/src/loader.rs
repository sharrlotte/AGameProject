use std::collections::HashSet;
use std::marker::PhantomData;

use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext, LoadState, UntypedAssetId};
use bevy::ecs::component::Mutable;
use bevy::prelude::*;
use serde::de::DeserializeOwned;
use thiserror::Error;

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

/// System set for initial `.ron` config synchronization during `GameState::Loading`.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConfigSyncSet;

/// Trait implemented by `.ron`-backed configuration resources.
pub trait RonConfig:
    Asset
    + Resource<Mutability = Mutable>
    + Clone
    + Default
    + DeserializeOwned
    + TypePath
    + Send
    + Sync
    + 'static
{
    /// Asset path relative to `game/core/assets`, e.g. `"config/arena.ron"`.
    const PATH: &'static str;
}

#[derive(Resource)]
struct ConfigHandle<T: Asset>(Handle<T>);

#[derive(Resource)]
struct RonConfigRegistered<T>(PhantomData<fn() -> T>);

#[derive(Resource, Default)]
struct PendingConfigs {
    pending: HashSet<UntypedAssetId>,
    failed: HashSet<UntypedAssetId>,
    finished: bool,
}

/// Extension trait for registering `.ron`-backed configuration types on a Bevy [`App`].
pub trait AppRonConfigExt {
    /// Registers asset loading, active resource, initial loading gate sync, and hot-reload for `T`.
    fn register_ron_config<T: RonConfig>(&mut self) -> &mut Self;
}

impl AppRonConfigExt for App {
    fn register_ron_config<T: RonConfig>(&mut self) -> &mut Self {
        if self.world().contains_resource::<RonConfigRegistered<T>>() {
            return self;
        }

        self.insert_resource(RonConfigRegistered::<T>(PhantomData))
            .init_resource::<PendingConfigs>()
            .init_asset::<T>()
            .register_asset_loader(RonAssetLoader::<T>::default())
            .init_resource::<T>()
            .add_systems(Startup, start_loading_config::<T>)
            .add_systems(
                Update,
                sync_initial_config::<T>
                    .in_set(ConfigSyncSet)
                    .run_if(in_state(GameState::Loading)),
            )
            .add_systems(
                Update,
                hot_reload_config::<T>
                    .before(SimUpdateSet)
                    .run_if(not(in_state(GameState::Loading))),
            )
    }
}

/// Coordinates initial `.ron` config loading and transitions from `GameState::Loading` to `Playing`.
pub struct DataPlugin;

impl Plugin for DataPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PendingConfigs>().add_systems(
            Update,
            finish_initial_config_load
                .after(ConfigSyncSet)
                .run_if(in_state(GameState::Loading)),
        );
    }
}

fn start_loading_config<T: RonConfig>(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut pending: ResMut<PendingConfigs>,
) {
    let handle = asset_server.load::<T>(T::PATH);
    pending.pending.insert(handle.id().untyped());
    commands.insert_resource(ConfigHandle(handle));
}

fn sync_initial_config<T: RonConfig>(
    handle: Option<Res<ConfigHandle<T>>>,
    asset_server: Res<AssetServer>,
    assets: Res<Assets<T>>,
    mut active: ResMut<T>,
    mut pending: ResMut<PendingConfigs>,
) {
    let Some(handle) = handle else {
        return;
    };
    let id = handle.0.id().untyped();
    if !pending.pending.contains(&id) {
        return;
    }

    if let Some(asset) = assets.get(&handle.0) {
        *active = asset.clone();
        pending.pending.remove(&id);
        pending.failed.remove(&id);
        return;
    }

    if let LoadState::Failed(err) = asset_server.load_state(&handle.0)
        && pending.failed.insert(id)
    {
        error!("Failed to load RON config '{}': {err:?}", T::PATH);
    }
}

fn finish_initial_config_load(
    mut pending: ResMut<PendingConfigs>,
    mut reloaded_writer: MessageWriter<ConfigHotReloadedMessage>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if !pending.finished && pending.pending.is_empty() {
        pending.finished = true;
        reloaded_writer.write(ConfigHotReloadedMessage);
        next_state.set(GameState::Playing);
    }
}

fn hot_reload_config<T: RonConfig>(
    mut events: MessageReader<AssetEvent<T>>,
    handle: Option<Res<ConfigHandle<T>>>,
    assets: Res<Assets<T>>,
    mut active: ResMut<T>,
    mut reloaded_writer: MessageWriter<ConfigHotReloadedMessage>,
) {
    let Some(handle) = handle else {
        return;
    };
    let mut updated = false;
    for event in events.read() {
        if let AssetEvent::Modified { id } = event
            && *id == handle.0.id()
            && let Some(asset) = assets.get(*id)
        {
            *active = asset.clone();
            updated = true;
        }
    }
    if updated {
        reloaded_writer.write(ConfigHotReloadedMessage);
    }
}

/// Resolves `game/core/assets/<T::PATH>` from any workspace crate and parses it with `ron`.
#[macro_export]
macro_rules! assert_ron_config_parses {
    ($ty:ty) => {{
        let path_rel = <$ty as $crate::RonConfig>::PATH;
        let assets_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .find_map(|dir| {
                let core_assets = dir.join("core").join("assets");
                if core_assets.join("config").exists() {
                    return Some(core_assets);
                }
                let direct_assets = dir.join("assets");
                if direct_assets.join("config").exists() {
                    return Some(direct_assets);
                }
                None
            })
            .expect("Could not locate game/core/assets directory");
        let path = assets_dir.join(path_rel);
        let contents = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("Failed to read {}: {err}", path.display()));
        ron::de::from_str::<$ty>(&contents)
            .unwrap_or_else(|err| panic!("Failed to parse {}: {err}", path.display()))
    }};
}
