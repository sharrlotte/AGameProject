use bevy::prelude::*;
use uuid::Uuid;

use crate::config::GameConfigValue;
use crate::content::block::BlockRegistry;
use crate::content::item::ItemRegistry;
use crate::quest::component::QuestCatalogValue;
use crate::quest::lib::{quest_rows, restore_tracker};
use crate::quest::QuestTracker;
use crate::resource::{AppNotice, DeployPayload, GameRng, SceneState, SessionClock};
use crate::save::design::{
    DesignCatalogSource, ShipCatalog, load_or_seed_designs, starter_designs, starter_fleet,
};

use crate::save::world::component::{
    ActiveStateSnapshot, AutoSaveTimer, ContinueWorldEvent, CreateSlotEvent,
    EditorReturnTarget, LoadSlotEvent, PendingExitAction, SaveWorldEvent, SlotSummary,
    StartQuickBattleEvent, WorldContext, WorldLoadData, WorldMeta, WorldStoreError,
};

use crate::save::world::snapshot::{
    BodySnapshot, CameraSave, ControlSave, apply_bodies,
    apply_camera, apply_control, apply_drops, apply_intel, capture_bodies, capture_camera,
    capture_control, capture_drops, capture_intel,
};

use crate::save::world::sqlite::{slot_path, SqliteWorldStore};
use crate::save::world::store::{InMemoryWorldStore, WorldStore, decode, encode, now_unix_ms};

pub fn init_world_session(
    mut context: ResMut<WorldContext>,
    mut source: ResMut<DesignCatalogSource>,
) {
    *context = WorldContext::default();
    *source = DesignCatalogSource::HomeSandbox;
}

pub fn handle_save_world_event(
    mut events: MessageReader<SaveWorldEvent>,
    world: &World,
    time: Res<Time>,
    clock: Res<SessionClock>,
    rng: Res<GameRng>,
    context: Res<WorldContext>,
    tracker: Res<QuestTracker>,
) {
    if events.read().next().is_none() {
        return;
    }

    if !context.is_save_slot() {
        return;
    }

    let active = context.store();

    let built = (|| -> Result<(WorldMeta, ActiveStateSnapshot), WorldStoreError> {
        let previous = active.load_meta().ok();
        let now = now_unix_ms();
        let meta = WorldMeta {
            name: previous
                .as_ref()
                .map_or_else(|| "World".to_string(), |m| m.name.clone()),
            seed: rng.seed(),
            play_time_secs: clock.elapsed(time.elapsed_secs()),
            created_at_ms: previous.as_ref().map_or(now, |m| m.created_at_ms),
            updated_at_ms: now,
            camera_blob: encode(&capture_camera(world))?,
            control_blob: encode(&capture_control(world))?,
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

    let quest_records = quest_rows(&tracker);
    if let Err(e) = active
        .save_world(&meta, &snapshot, &quest_records)
        .and_then(|()| active.flush())
    {
        error!("Save failed: {e}");
    }
}

pub fn handle_load_slot(
    mut events: MessageReader<LoadSlotEvent>,
    registry: Res<BlockRegistry>,
    mut context: ResMut<WorldContext>,
    mut source: ResMut<DesignCatalogSource>,
    mut catalog: ResMut<ShipCatalog>,
    mut deploy: ResMut<DeployPayload>,
    config: Res<GameConfigValue>,
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
        let invalid = designs.iter().find_map(|design| {
            design.validate(&registry).err().map(|errors| (design.name.clone(), errors))
        });

        if let Some((name, errors)) = invalid {
            let reason = format!("invalid design {name}: {}", errors.join("; "));
            warn!("Load failed: {reason}");
            notice.show(format!("Load failed: {reason}"));
            continue;
        }

        let active_slot_id = opened.slot_id().unwrap_or(slot_id);
        let active_path = opened.path().to_path_buf();
        if active_slot_id != slot_id {
            notice.show("World migrated to new save slot");
        }

        deploy.fleet = starter_fleet(&config.0, &designs);
        catalog.replace_all(designs);
        *source = DesignCatalogSource::ActiveWorldDb;
        *context = WorldContext::Restoring {
            slot_id: active_slot_id,
            path: active_path,
            store: Box::new(opened),
        };
        scene.set(SceneState::Loading);
    }
}

pub fn handle_create_slot(
    mut events: MessageReader<CreateSlotEvent>,
    mut rng: ResMut<GameRng>,
    mut context: ResMut<WorldContext>,
    mut source: ResMut<DesignCatalogSource>,
    mut catalog: ResMut<ShipCatalog>,
    mut deploy: ResMut<DeployPayload>,
    config: Res<GameConfigValue>,
    mut scene: ResMut<NextState<SceneState>>,
    mut notice: ResMut<AppNotice>,
    time: Res<Time>,
    mut clock: ResMut<SessionClock>,
    mut tracker: ResMut<QuestTracker>,
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

        *rng = GameRng::new(seed);
        deploy.fleet = starter_fleet(&config.0, &designs);
        catalog.replace_all(designs);
        *source = DesignCatalogSource::ActiveWorldDb;
        *context = WorldContext::SaveSlot { slot_id, path, store: Box::new(created) };
        clock.reset();
        clock.resume(time.elapsed_secs(), 0.0);
        *tracker = QuestTracker::default();
        scene.set(SceneState::Game);
    }
}

fn resolve_slot_id(context: &WorldContext, slots: &[SlotSummary]) -> Option<Uuid> {
    match context {
        WorldContext::SaveSlot { slot_id, .. } | WorldContext::Restoring { slot_id, .. } => Some(*slot_id),
        WorldContext::QuickBattle { .. } => slots
            .iter()
            .max_by_key(|slot| slot.updated_at_ms)
            .map(|slot| slot.slot_id),
    }
}

pub fn handle_continue_world(
    mut events: MessageReader<ContinueWorldEvent>,
    registry: Res<BlockRegistry>,
    config: Res<GameConfigValue>,
    mut context: ResMut<WorldContext>,
    mut source: ResMut<DesignCatalogSource>,
    mut catalog: ResMut<ShipCatalog>,
    mut deploy: ResMut<DeployPayload>,
    mut notice: ResMut<AppNotice>,
    mut scene: ResMut<NextState<SceneState>>,
    mut creates: MessageWriter<CreateSlotEvent>,
) {
    if events.read().next().is_none() {
        return;
    }

    let slots = context.store().list_slots().unwrap_or_default();
    let slot_id = resolve_slot_id(&context, &slots);

    let Some(slot_id) = slot_id else {
        creates.write(CreateSlotEvent { name: format!("World {}", now_unix_ms()) });
        return;
    };

    let path = slot_path(slot_id);
    let opened = match SqliteWorldStore::open(&path) {
        Ok(opened) => opened,
        Err(e) => {
            warn!("Continue failed: {e}");
            notice.show(format!("Continue failed: {e}"));
            return;
        }
    };

    let designs = match opened.list_designs() {
        Ok(designs) => designs,
        Err(e) => {
            warn!("Continue failed: {e}");
            notice.show(format!("Continue failed: {e}"));
            return;
        }
    };
    if let Some((name, errors)) = designs.iter().find_map(|design| {
        design.validate(&registry).err().map(|errors| (design.name.clone(), errors))
    }) {
        let reason = format!("invalid design {name}: {}", errors.join("; "));
        warn!("Continue failed: {reason}");
        notice.show(format!("Continue failed: {reason}"));
        return;
    }
    if let Err(e) = load_world_data(&opened, &registry).and_then(|data| {
        validate_references(&data)?;
        Ok(())
    }) {
        warn!("Continue failed: {e}");
        notice.show(format!("Continue failed: {e}"));
        return;
    }

    deploy.fleet = starter_fleet(&config.0, &designs);
    catalog.replace_all(designs);
    *source = DesignCatalogSource::ActiveWorldDb;
    *context = WorldContext::Restoring { slot_id, path, store: Box::new(opened) };
    scene.set(SceneState::Game);
}

pub fn handle_quick_battle(
    mut events: MessageReader<StartQuickBattleEvent>,
    registry: Res<BlockRegistry>,
    mut rng: ResMut<GameRng>,
    mut context: ResMut<WorldContext>,
    mut source: ResMut<DesignCatalogSource>,
    mut catalog: ResMut<ShipCatalog>,
    mut deploy: ResMut<DeployPayload>,
    config: Res<GameConfigValue>,
    mut scene: ResMut<NextState<SceneState>>,
    time: Res<Time>,
    mut clock: ResMut<SessionClock>,
) {
    if events.read().next().is_none() {
        return;
    }

    let designs = load_or_seed_designs(&registry);
    *rng = GameRng::new(rand::random::<u64>());
    deploy.fleet = starter_fleet(&config.0, &designs);
    catalog.replace_all(designs);
    *source = DesignCatalogSource::HomeSandbox;
    *context = WorldContext::QuickBattle { store: Box::new(InMemoryWorldStore::default()) };
    clock.reset();
    clock.resume(time.elapsed_secs(), 0.0);
    scene.set(SceneState::Game);
}

pub fn begin_loaded_game(context: Res<WorldContext>, mut next_scene: ResMut<NextState<SceneState>>) {
    if context.is_restoring() {
        next_scene.set(SceneState::Game);
    }
}

fn load_world_data(active: &dyn WorldStore, registry: &BlockRegistry) -> Result<WorldLoadData, WorldStoreError> {
    let meta = active.load_meta()?;
    let state = active.load_active_state()?;
    let seed = meta.seed;
    let camera_save = decode::<CameraSave>(&meta.camera_blob)?;
    let control_save = decode::<ControlSave>(&meta.control_blob)?;
    let bodies: Vec<BodySnapshot> = decode(&state.bodies_blob)?;

    for body in &bodies {
        let result = match body {
            BodySnapshot::Ship(ship) => ship.validate(registry),
            BodySnapshot::Debris(debris) => debris.validate(registry),
        };
        if let Err(errors) = result {
            return Err(WorldStoreError::InvalidData(errors.join("; ")));
        }
    }

    let ship_ids = bodies
        .iter()
        .filter_map(|body| match body {
            BodySnapshot::Ship(ship) => Some(ship.id),
            BodySnapshot::Debris(_) => None,
        })
        .collect::<std::collections::HashSet<_>>();

    if ship_ids.len() != bodies.iter().filter(|b| matches!(b, BodySnapshot::Ship(_))).count() {
        return Err(WorldStoreError::InvalidData("duplicate ship id in save".to_string()));
    }

    let drops = decode(&state.drops_blob)?;
    let intel = decode(&state.intel_blob)?;
    let quests = active.load_quests()?;

    Ok(WorldLoadData {
        seed,
        play_time_secs: meta.play_time_secs,
        camera_save,
        control_save,
        bodies,
        ship_ids,
        drops,
        intel,
        quests,
    })
}

fn validate_references(data: &WorldLoadData) -> Result<(), WorldStoreError> {
    let missing = |id: Uuid| WorldStoreError::InvalidData(format!("dangling ship reference {id}"));
    if let Some(id) = data.camera_save.following
        && !data.ship_ids.contains(&id)
    {
        return Err(missing(id));
    }

    if let Some(id) = data.control_save.controlling
        && !data.ship_ids.contains(&id)
    {
        return Err(missing(id));
    }

    for plan in &data.intel {
        for seen in &plan.last_known {
            if !data.ship_ids.contains(&seen.target_id) {
                return Err(missing(seen.target_id));
            }
        }
    }
    
    for body in &data.bodies {
        if let BodySnapshot::Ship(ship) = body
            && let Some(id) = ship.brain.current.target_uuid()
            && !data.ship_ids.contains(&id)
        {
            return Err(missing(id));
        }
    }
    Ok(())
}

fn apply_world_data(
    commands: &mut Commands,
    data: WorldLoadData,
    registry: &BlockRegistry,
    config: &GameConfigValue,
    camera: &mut Query<(&mut Transform, &mut Projection), With<Camera2d>>,
    items: Option<&ItemRegistry>,
    tracker: &mut QuestTracker,
    catalog: &QuestCatalogValue,
) -> Result<(), WorldStoreError> {
    let ship_map = apply_bodies(commands, data.bodies, registry, config)?;
    apply_intel(commands, data.intel, &ship_map);
    apply_control(commands, &data.control_save, &ship_map);
    apply_camera(camera, &data.camera_save, config, &ship_map, commands);
    apply_drops(commands, data.drops, config, items);
    *tracker = restore_tracker(&data.quests, catalog);
    Ok(())
}

pub fn spawn_loaded_world(
    mut commands: Commands,
    mut context: ResMut<WorldContext>,
    registry: Res<BlockRegistry>,
    items: Option<Res<ItemRegistry>>,
    config: Res<GameConfigValue>,
    time: Res<Time>,
    mut notice: ResMut<AppNotice>,
    mut scene: ResMut<NextState<SceneState>>,
    mut camera: Query<(&mut Transform, &mut Projection), With<Camera2d>>,
    transients: Query<Entity, With<DespawnOnExit<SceneState>>>,
    mut rng: ResMut<GameRng>,
    mut clock: ResMut<SessionClock>,
    mut tracker: ResMut<QuestTracker>,
    catalog: Res<QuestCatalogValue>,
) {
    let Some((slot_id, path, store)) = context.take_restoring() else {
        return;
    };

    let data = match load_world_data(store.as_ref(), &registry) {
        Ok(data) => data,
        Err(e) => {
            *context = WorldContext::SaveSlot { slot_id, path, store };
            warn!("Failed to read save slot: {e}");
            notice.show(format!("Load failed: {e}"));
            scene.set(SceneState::Home);
            return;
        }
    };

    if let Err(e) = validate_references(&data) {
        *context = WorldContext::SaveSlot { slot_id, path, store };
        warn!("Failed to restore save slot: {e}");
        notice.show(format!("Load failed: {e}"));
        scene.set(SceneState::Home);
        return;
    }

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
        &mut tracker,
        &catalog,
    ) {
        *context = WorldContext::SaveSlot { slot_id, path, store };
        warn!("Failed to restore save slot: {e}");
        notice.show(format!("Load failed: {e}"));
        scene.set(SceneState::Home);
        return;
    }

    *context = WorldContext::SaveSlot { slot_id, path, store };
    *rng = GameRng::new(seed);
    clock.resume(time.elapsed_secs(), play_time_secs);
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
    context: Res<WorldContext>,
    mut timer: Option<ResMut<AutoSaveTimer>>,
    mut events: MessageWriter<SaveWorldEvent>,
) {
    if !context.is_save_slot() {
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
    registry: Res<BlockRegistry>,
) {
    *source = DesignCatalogSource::HomeSandbox;
    catalog.designs = load_or_seed_designs(&registry);
}

#[cfg(test)]
mod tests {
    use super::*;
    use SlotSummary;
    use std::path::PathBuf;

    fn slot(id: Uuid, updated: u64) -> SlotSummary {
        SlotSummary {
            slot_id: id,
            name: "World".to_string(),
            seed: 1,
            play_time_secs: 0.0,
            created_at_ms: 0,
            updated_at_ms: updated,
        }
    }

    #[test]
    fn active_slot_wins_over_newer_slot() {
        let active = Uuid::now_v7();
        let other = Uuid::now_v7();
        let context = WorldContext::SaveSlot {
            slot_id: active,
            path: PathBuf::new(),
            store: Box::new(InMemoryWorldStore::default()),
        };
        let slots = [slot(active, 10), slot(other, 999)];
        assert_eq!(resolve_slot_id(&context, &slots), Some(active));
    }

    #[test]
    fn quick_battle_uses_most_recent_slot() {
        let older = Uuid::now_v7();
        let newer = Uuid::now_v7();
        let context = WorldContext::default();
        let slots = [slot(older, 10), slot(newer, 20)];
        assert_eq!(resolve_slot_id(&context, &slots), Some(newer));
        assert_eq!(resolve_slot_id(&context, &[]), None);
    }

    #[test]
    fn world_context_transitions_restore_to_save_slot() {
        let id = Uuid::now_v7();
        let mut context = WorldContext::Restoring {
            slot_id: id,
            path: PathBuf::new(),
            store: Box::new(InMemoryWorldStore::default()),
        };
        assert!(context.is_restoring());
        let (slot_id, _, store) = context.take_restoring().expect("restoring");
        context = WorldContext::SaveSlot { slot_id, path: PathBuf::new(), store };
        assert!(!context.is_restoring());
        assert_eq!(context.slot_id(), Some(id));
    }
}
