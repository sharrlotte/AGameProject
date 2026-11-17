use std::time::{SystemTime, UNIX_EPOCH};

use bevy::prelude::*;

use crate::config::GameConfigValue;
use crate::content::block::BlockRegistry;
use crate::content::item::ItemRegistry;
use crate::mission::{BattleOutcome, BattlePhase, BattleResult};
use crate::resource::{AppNotice, BattleMode, GameRng, PendingLoad, SceneState};
use crate::save::document::{read_document, write_document};

use super::component::{
    BattleSaveState, CaptureContext, LoadWorldEvent, PendingExitAction, SaveResult, SaveWorldEvent,
    WorldSave,
};
use super::lib::{battle_save_path, has_battle_save};

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn parse_outcome(text: &Option<String>) -> Option<BattleOutcome> {
    match text.as_deref() {
        Some("Victory") => Some(BattleOutcome::Victory),
        Some("Defeat") => Some(BattleOutcome::Defeat),
        _ => None,
    }
}

pub fn init_battle_save_state(mut state: ResMut<BattleSaveState>) {
    state.has_save = has_battle_save();
}

pub fn handle_save_world_event(
    mut events: MessageReader<SaveWorldEvent>,
    mut commands: Commands,
    world: &World,
    time: Res<Time>,
    battle: Res<BattleResult>,
    game_rng: Res<GameRng>,
    mode: Res<BattleMode>,
) {
    if events.read().next().is_none() {
        return;
    }
    if *mode != BattleMode::Campaign {
        return;
    }

    let elapsed_secs = battle
        .started_at
        .map(|start| (time.elapsed_secs() - start).max(0.0))
        .unwrap_or(battle.duration_secs);
    let ctx = CaptureContext {
        world,
        mission: &battle.mission_name,
        elapsed_secs,
        outcome: battle.outcome.map(|o| format!("{o:?}")),
        saved_at_unix_ms: now_unix_ms(),
        seed: game_rng.seed(),
    };
    let save = WorldSave::capture(&ctx);

    match write_document(&battle_save_path(), &save) {
        Ok(()) => {
            commands.insert_resource(BattleSaveState { has_save: true });
        }
        Err(e) => {
            commands.insert_resource(SaveResult {
                message: Some(format!("Save failed: {e}")),
            });
        }
    }
}

pub fn flush_pending_exit_action(
    mut action: ResMut<PendingExitAction>,
    mut next_scene: ResMut<NextState<SceneState>>,
) {
    match *action {
        PendingExitAction::None => {}
        PendingExitAction::Home => {
            *action = PendingExitAction::None;
            next_scene.set(SceneState::Home);
        }
    }
}

pub fn report_save_result(mut result: ResMut<SaveResult>, mut notice: ResMut<AppNotice>) {
    if let Some(message) = result.message.take() {
        notice.show(message);
    }
}

pub fn handle_load_request(
    mut events: MessageReader<LoadWorldEvent>,
    mut pending: ResMut<PendingLoad>,
    mut mode: ResMut<BattleMode>,
    mut next_scene: ResMut<NextState<SceneState>>,
) {
    if events.read().next().is_some() {
        *mode = BattleMode::Campaign;
        pending.active = true;
        next_scene.set(SceneState::Loading);
    }
}

pub fn begin_loaded_game(pending: Res<PendingLoad>, mut next_scene: ResMut<NextState<SceneState>>) {
    if pending.is_pending() {
        next_scene.set(SceneState::Game);
    }
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
    mut scene_state: ResMut<NextState<SceneState>>,
    mut camera: Query<(&mut Transform, &mut Projection), With<Camera2d>>,
    transients: Query<Entity, With<DespawnOnExit<SceneState>>>,
) {
    if !pending.is_pending() {
        return;
    }

    commands.insert_resource(PendingLoad::default());

    let save: WorldSave = match read_document(&battle_save_path()) {
        Ok(save) => save,
        Err(e) => {
            warn!("Failed to read battle save: {e}");
            notice.show(format!("Load failed: {e}"));
            scene_state.set(SceneState::Home);
            return;
        }
    };

    for entity in &transients {
        commands.entity(entity).despawn();
    }

    if let Err(e) = save
        .clone()
        .restore(&mut commands, &registry, &config, items.as_deref())
    {
        warn!("Failed to restore battle save: {e}");
        notice.show(format!("Load failed: {e}"));
        scene_state.set(SceneState::Home);
        return;
    }

    for (mut transform, mut projection) in &mut camera {
        transform.translation.x = save.camera.x;
        transform.translation.y = save.camera.y;
        if let Projection::Orthographic(ortho) = &mut *projection {
            ortho.scale = save.camera.zoom.clamp(config.0.min_zoom, config.0.max_zoom);
        }
    }

    commands.insert_resource(GameRng::new(save.rng.seed));
    let outcome = parse_outcome(&save.battle.outcome);
    *battle = BattleResult {
        started_at: Some(time.elapsed_secs() - save.battle.elapsed_secs),
        outcome,
        duration_secs: save.battle.elapsed_secs,
        mission_name: save.battle.mission,
    };
    if outcome.is_some() {
        phase.set(BattlePhase::Ended);
    } else {
        phase.set(BattlePhase::Active);
    }
}
