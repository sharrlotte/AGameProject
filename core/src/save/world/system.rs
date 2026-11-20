use bevy::prelude::*;
use uuid::Uuid;

use crate::config::GameConfigValue;
use crate::content::block::BlockRegistry;
use crate::content::item::ItemRegistry;
use crate::mission::{BattlePhase, BattleResult};
use crate::resource::{AppNotice, BattleMode, DeployPayload, GameRng, PendingLoad, SceneState};
use crate::save::design::{
    DesignCatalogSource, ShipCatalog, load_or_seed_designs, starter_designs, starter_fleet,
};

use crate::save::world::component::{
    ActiveStateSnapshot, ActiveWorldStore, AutoSaveTimer, CreateSlotEvent, EditorReturnTarget,
    LoadSlotEvent, PendingExitAction, SaveWorldEvent, StartQuickBattleEvent, WorldMeta,
    WorldSession, WorldStoreError,
};

use crate::save::world::snapshot::{
    BodySnapshot, CameraSave, ControlSave, DropSave, RngSave, TeamIntelSave, apply_bodies,
    apply_camera, apply_control, apply_drops, apply_intel, capture_bodies, capture_camera,
    capture_control, capture_drops, capture_intel,
};

use crate::save::world::sqlite::{slot_path, SqliteWorldStore};
use crate::save::world::store::{InMemoryWorldStore, WorldStore, decode, encode, now_unix_ms};

pub fn init_world_session(
    mut session: ResMut<WorldSession>,
    mut store: ResMut<ActiveWorldStore>,
    mut source: ResMut<DesignCatalogSource>,
) {
    *session = WorldSession::QuickBattle;
    store.0 = Some(Box::new(InMemoryWorldStore::default()));
    *source = DesignCatalogSource::HomeSandbox;
}

pub fn handle_save_world_event(
    mut events: MessageReader<SaveWorldEvent>,
    world: &World,
    time: Res<Time>,
    battle: Res<BattleResult>,
    rng: Res<GameRng>,
    session: Res<WorldSession>,
    store: Res<ActiveWorldStore>,
) {
    if events.read().next().is_none() {
        return;
    }

    if !matches!(*session, WorldSession::SaveSlot { .. }) {
        return;
    }

    let Some(active) = store.0.as_ref() else {
        error!("Save failed: no active world store");
        return;
    };

    let built = (|| -> Result<(WorldMeta, ActiveStateSnapshot), WorldStoreError> {
        let previous = active.load_meta().ok();
        let now = now_unix_ms();
        let meta = WorldMeta {
            name: previous
                .as_ref()
                .map_or_else(|| "World".to_string(), |m| m.name.clone()),
            seed: rng.seed(),
            play_time_secs: battle_duration(&time, &battle),
            created_at_ms: previous.as_ref().map_or(now, |m| m.created_at_ms),
            updated_at_ms: now,
            camera_blob: encode(&capture_camera(world))?,
            control_blob: encode(&capture_control(world))?,
            rng_blob: encode(&RngSave { seed: rng.seed() })?,
        };
        let snapshot = ActiveStateSnapshot {
            bodies_blob: encode(&capture_bodies(world))?,
            drops_blob: encode(&capture_drops(world))?,
            intel_blob: encode(&capture_intel(world))?,
        };
        Ok((meta, snapshot))
    })();

    let (meta, snapshot) = match built {
        Ok(built) => built,
        Err(e) => {
            error!("Save failed: {e}");
            return;
        }
    };

    if let Err(e) = active.save_world(&meta, &snapshot).and_then(|()| active.flush()) {
        error!("Save failed: {e}");
    }
}

pub fn handle_load_slot(
    mut events: MessageReader<LoadSlotEvent>,
    mut store: ResMut<ActiveWorldStore>,
    mut session: ResMut<WorldSession>,
    mut source: ResMut<DesignCatalogSource>,
    mut catalog: ResMut<ShipCatalog>,
    mut deploy: ResMut<DeployPayload>,
    config: Res<GameConfigValue>,
    mut mode: ResMut<BattleMode>,
    mut pending: ResMut<PendingLoad>,
    mut scene: ResMut<NextState<SceneState>>,
    mut notice: ResMut<AppNotice>,
) {
    for event in events.read() {
        let slot_id = event.slot.slot_id;
        let path = slot_path(slot_id);
        let opened = match SqliteWorldStore::open(&path) {
            Ok(opened) => opened,
            Err(e) => {
                warn!("Failed to open save slot {}: {e}", path.display());
                notice.show(format!("Load failed: {e}"));
                continue;
            }
        };

        let designs = match opened.list_designs() {
            Ok(designs) => designs,
            Err(e) => {
                warn!("Failed to read save slot designs: {e}");
                notice.show(format!("Load failed: {e}"));
                continue;
            }
        };

        let active_slot_id = opened.slot_id().unwrap_or(slot_id);
        let active_path = opened.path().to_path_buf();
        if active_slot_id != slot_id {
            notice.show("World migrated to new save slot");
        }

        deploy.fleet = starter_fleet(&config.0, &designs);
        catalog.replace_all(designs);
        *source = DesignCatalogSource::ActiveWorldDb;
        store.0 = Some(Box::new(opened));
        *session = WorldSession::SaveSlot {
            slot_id: active_slot_id,
            path: active_path,
        };
        *mode = BattleMode::Campaign;
        pending.active = true;
        scene.set(SceneState::Loading);
    }
}

pub fn handle_create_slot(
    mut events: MessageReader<CreateSlotEvent>,
    mut store: ResMut<ActiveWorldStore>,
    mut session: ResMut<WorldSession>,
    mut source: ResMut<DesignCatalogSource>,
    mut catalog: ResMut<ShipCatalog>,
    mut deploy: ResMut<DeployPayload>,
    config: Res<GameConfigValue>,
    mut mode: ResMut<BattleMode>,
    mut pending: ResMut<PendingLoad>,
    mut scene: ResMut<NextState<SceneState>>,
    mut notice: ResMut<AppNotice>,
) {
    for event in events.read() {
        let slot_id = Uuid::now_v7();
        let path = slot_path(slot_id);
        let seed = rand::random::<u64>();
        let now = now_unix_ms();
        let designs = starter_designs();

        let built = (|| -> Result<WorldMeta, WorldStoreError> {
            Ok(WorldMeta {
                name: event.name.clone(),
                seed,
                play_time_secs: 0.0,
                created_at_ms: now,
                updated_at_ms: now,
                camera_blob: encode(&CameraSave::default())?,
                control_blob: encode(&ControlSave::default())?,
                rng_blob: encode(&RngSave { seed })?,
            })
        })();

        let meta = match built {
            Ok(meta) => meta,
            Err(e) => {
                warn!("Failed to encode new world: {e}");
                notice.show(format!("New World failed: {e}"));
                continue;
            }
        };

        let created = match SqliteWorldStore::create(&path, &meta, &designs) {
            Ok(created) => created,
            Err(e) => {
                warn!("Failed to create save slot {}: {e}", path.display());
                notice.show(format!("New World failed: {e}"));
                continue;
            }
        };

        deploy.fleet = starter_fleet(&config.0, &designs);
        catalog.replace_all(designs);
        *source = DesignCatalogSource::ActiveWorldDb;
        store.0 = Some(Box::new(created));
        *session = WorldSession::SaveSlot { slot_id, path };
        *mode = BattleMode::Campaign;
        pending.active = false;
        scene.set(SceneState::Game);
    }
}

pub fn handle_quick_battle(
    mut events: MessageReader<StartQuickBattleEvent>,
    mut store: ResMut<ActiveWorldStore>,
    mut session: ResMut<WorldSession>,
    mut source: ResMut<DesignCatalogSource>,
    mut catalog: ResMut<ShipCatalog>,
    mut deploy: ResMut<DeployPayload>,
    config: Res<GameConfigValue>,
    mut mode: ResMut<BattleMode>,
    mut pending: ResMut<PendingLoad>,
    mut scene: ResMut<NextState<SceneState>>,
) {
    if events.read().next().is_none() {
        return;
    }

    let designs = load_or_seed_designs();
    deploy.fleet = starter_fleet(&config.0, &designs);
    catalog.replace_all(designs);
    *source = DesignCatalogSource::HomeSandbox;
    store.0 = Some(Box::new(InMemoryWorldStore::default()));
    *session = WorldSession::QuickBattle;
    *mode = BattleMode::QuickBattle;
    pending.active = false;
    scene.set(SceneState::Game);
}

pub fn begin_loaded_game(pending: Res<PendingLoad>, mut next_scene: ResMut<NextState<SceneState>>) {
    if pending.is_pending() {
        next_scene.set(SceneState::Game);
    }
}

struct WorldLoadData {
    seed: u64,
    play_time_secs: f32,
    camera_save: CameraSave,
    control_save: ControlSave,
    bodies: Vec<BodySnapshot>,
    drops: Vec<DropSave>,
    intel: Vec<TeamIntelSave>,
}

fn load_world_data(active: &dyn WorldStore) -> Result<WorldLoadData, WorldStoreError> {
    let meta = active.load_meta()?;
    let state = active.load_active_state()?;
    let seed = decode::<RngSave>(&meta.rng_blob)?.seed;
    let camera_save = decode::<CameraSave>(&meta.camera_blob)?;
    let control_save = decode::<ControlSave>(&meta.control_blob)?;
    let bodies = decode(&state.bodies_blob)?;
    let drops = decode(&state.drops_blob)?;
    let intel = decode(&state.intel_blob)?;

    Ok(WorldLoadData {
        seed,
        play_time_secs: meta.play_time_secs,
        camera_save,
        control_save,
        bodies,
        drops,
        intel,
    })
}

fn apply_world_data(
    commands: &mut Commands,
    data: WorldLoadData,
    registry: &BlockRegistry,
    config: &GameConfigValue,
    camera: &mut Query<(&mut Transform, &mut Projection), With<Camera2d>>,
    items: Option<&ItemRegistry>,
) -> Result<(), WorldStoreError> {
    let ship_map = apply_bodies(commands, data.bodies, registry, config)?;
    apply_intel(commands, data.intel, &ship_map);
    apply_control(commands, &data.control_save, &ship_map);
    apply_camera(camera, &data.camera_save, config, &ship_map, commands);
    apply_drops(commands, data.drops, config, items);
    Ok(())
}

pub fn spawn_loaded_world(
    mut commands: Commands,
    pending: Res<PendingLoad>,
    registry: Res<BlockRegistry>,
    items: Option<Res<ItemRegistry>>,
    config: Res<GameConfigValue>,
    time: Res<Time>,
    mut battle: ResMut<BattleResult>,
    mut phase: ResMut<NextState<BattlePhase>>,
    mut notice: ResMut<AppNotice>,
    mut scene: ResMut<NextState<SceneState>>,
    mut camera: Query<(&mut Transform, &mut Projection), With<Camera2d>>,
    transients: Query<Entity, With<DespawnOnExit<SceneState>>>,
    store: Res<ActiveWorldStore>,
    mut rng: ResMut<GameRng>,
) {
    if !pending.is_pending() {
        return;
    }

    commands.insert_resource(PendingLoad::default());

    let Some(active) = store.0.as_ref() else {
        warn!("Load failed: no active world store");
        notice.show("Load failed: no active world store");
        scene.set(SceneState::Home);
        return;
    };

    let data = match load_world_data(active.as_ref()) {
        Ok(data) => data,
        Err(e) => {
            warn!("Failed to read save slot: {e}");
            notice.show(format!("Load failed: {e}"));
            scene.set(SceneState::Home);
            return;
        }
    };

    for entity in &transients {
        commands.entity(entity).despawn();
    }

    let seed = data.seed;
    let play_time_secs = data.play_time_secs;

    if let Err(e) = apply_world_data(
        &mut commands,
        data,
        &registry,
        &config,
        &mut camera,
        items.as_deref(),
    ) {
        warn!("Failed to restore save slot: {e}");
        notice.show(format!("Load failed: {e}"));
        scene.set(SceneState::Home);
        return;
    }

    *rng = GameRng::new(seed);
    let mission_name = battle.mission_name.clone();

    *battle = BattleResult {
        started_at: Some((time.elapsed_secs() - play_time_secs).max(0.0)),
        outcome: None,
        duration_secs: play_time_secs,
        mission_name,
    };
    phase.set(BattlePhase::Active);
}

pub fn reset_auto_save_timer(mut commands: Commands, config: Res<GameConfigValue>) {
    let interval = config.0.auto_save_interval_secs.max(1.0);
    commands.insert_resource(AutoSaveTimer(Timer::from_seconds(
        interval,
        TimerMode::Repeating,
    )));
}

pub fn tick_auto_save(
    time: Res<Time>,
    session: Res<WorldSession>,
    mut timer: Option<ResMut<AutoSaveTimer>>,
    mut events: MessageWriter<SaveWorldEvent>,
) {
    if !matches!(*session, WorldSession::SaveSlot { .. }) {
        return;
    }
    let Some(ref mut timer) = timer else {
        return;
    };
    if timer.0.tick(time.delta()).just_finished() {
        events.write(SaveWorldEvent);
    }
}

pub fn flush_pending_exit_action(
    mut action: ResMut<PendingExitAction>,
    mut next_scene: ResMut<NextState<SceneState>>,
    mut editor_target: ResMut<EditorReturnTarget>,
) {
    match *action {
        PendingExitAction::None => {}
        PendingExitAction::Home => {
            *action = PendingExitAction::None;
            *editor_target = EditorReturnTarget::Home;
            next_scene.set(SceneState::Home);
        }
    }
}

pub fn reset_catalog_on_home(
    mut source: ResMut<DesignCatalogSource>,
    mut catalog: ResMut<ShipCatalog>,
) {
    *source = DesignCatalogSource::HomeSandbox;
    catalog.designs = load_or_seed_designs();
}

fn battle_duration(time: &Time, battle: &BattleResult) -> f32 {
    battle
        .started_at
        .map(|start| (time.elapsed_secs() - start).max(0.0))
        .unwrap_or(battle.duration_secs)
}
