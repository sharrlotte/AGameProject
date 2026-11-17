use bevy::prelude::*;
use directories::ProjectDirs;
use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::config::GameConfigValue;
use crate::content::block::BlockRegistry;
use crate::mission::{BattleOutcome, BattlePhase, BattleResult};
use crate::resource::{AppNotice, CameraTarget, ControlledShip, GameRng, PendingLoad, SceneState};
use crate::save::document::{read_document, write_document};
use crate::save::schema::*;
use crate::save::snapshot::{CaptureContext, CaptureSnapshot, RestoreSnapshot};
use crate::simulation::SimulationSet;

#[derive(Message)]
pub struct SaveWorldEvent {
    pub slot: u8,
}

/// Request a world load. Handled as a state transition so the outgoing scene
/// is despawned by `DespawnOnExit` before the snapshot is spawned.
#[derive(Message)]
pub struct LoadWorldEvent {
    pub slot: u8,
}

pub fn slot_path(slot: u8) -> PathBuf {
    data_dir().join("saves").join(format!("slot_{slot}.ron"))
}

pub fn data_dir() -> PathBuf {
    ProjectDirs::from("com", "somerandomdev", "agameproject")
        .map(|dirs| dirs.data_dir().to_path_buf())
        .unwrap_or_else(|| {
            error!("Failed to resolve project directory, falling back to local ./agameproject");
            PathBuf::from("./agameproject")
        })
}

/// Metadata for all five slots, read from the same document used for loading.
pub fn list_slots() -> Vec<SlotInfo> {
    (1..=SAVE_SLOTS as u8)
        .map(|slot| match read_document::<WorldSave>(&slot_path(slot)) {
            Ok(save) => SlotInfo {
                slot,
                occupied: true,
                saved_at_unix_ms: save.saved_at_unix_ms,
                mission: save.battle.mission,
                ship_count: save.ships.len(),
                elapsed_secs: save.battle.elapsed_secs,
            },
            Err(_) => SlotInfo {
                slot,
                occupied: false,
                ..default()
            },
        })
        .collect()
}

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

pub fn handle_save_world_event(
    mut events: MessageReader<SaveWorldEvent>,
    world: &World,
    time: Res<Time>,
    battle: Res<BattleResult>,
    game_rng: Res<GameRng>,
    mut notice: ResMut<AppNotice>,
) {
    for event in events.read() {
        let slot = event.slot.clamp(1, SAVE_SLOTS as u8);
        let elapsed_secs = battle
            .started_at
            .map(|start| (time.elapsed_secs() - start).max(0.0))
            .unwrap_or(battle.duration_secs);
        let ctx = CaptureContext {
            world,
            slot,
            mission: &battle.mission_name,
            elapsed_secs,
            outcome: battle.outcome.map(|o| format!("{o:?}")),
            saved_at_unix_ms: now_unix_ms(),
            seed: game_rng.seed(),
        };
        let save = WorldSave::capture(&ctx, Entity::PLACEHOLDER);

        match write_document(&slot_path(slot), &save) {
            Ok(()) => notice.show(format!("Saved slot {slot}")),
            Err(e) => notice.show(format!("Save failed: {e}")),
        }
    }
}

/// Convert a load request into a scene transition. The snapshot is spawned on
/// the next `SceneState::Game` entry, after the outgoing scene has been torn
/// down by `DespawnOnExit(SceneState::Game)`.
pub fn handle_load_request(
    mut events: MessageReader<LoadWorldEvent>,
    mut pending: ResMut<PendingLoad>,
    mut next_scene: ResMut<NextState<SceneState>>,
) {
    for event in events.read() {
        pending.slot = Some(event.slot.clamp(1, SAVE_SLOTS as u8));
        next_scene.set(SceneState::Loading);
    }
}

/// Move from `Loading` into `Game` once a load has been requested.
pub fn begin_loaded_game(
    pending: Res<PendingLoad>,
    mut next_scene: ResMut<NextState<SceneState>>,
) {
    if pending.is_pending() {
        next_scene.set(SceneState::Game);
    }
}

/// Spawn the requested snapshot when `Game` is entered. Runs alongside
/// `setup_battle`, which skips itself while a load is pending.
pub fn spawn_loaded_world(
    mut commands: Commands,
    pending: Res<PendingLoad>,
    registry: Res<BlockRegistry>,
    config: Res<GameConfigValue>,
    time: Res<Time>,
    mut battle: ResMut<BattleResult>,
    mut phase: ResMut<NextState<BattlePhase>>,
    mut notice: ResMut<AppNotice>,
    mut scene_state: ResMut<NextState<SceneState>>,
    mut camera: Query<(&mut Transform, &mut Projection), With<Camera2d>>,
    transients: Query<Entity, With<DespawnOnExit<SceneState>>>,
) {
    let Some(slot) = pending.slot else {
        return;
    };

    // Deferred so other OnEnter(Game) systems still observe the pending flag
    // this frame and skip mission setup.
    commands.insert_resource(PendingLoad::default());
    commands.insert_resource(CameraTarget::default());
    commands.insert_resource(ControlledShip::default());

    let save: WorldSave = match read_document(&slot_path(slot)) {
        Ok(save) => save,
        Err(e) => {
            warn!("Failed to read slot {slot}: {e}");
            notice.show(format!("Load failed: {e}"));
            scene_state.set(SceneState::Home);
            return;
        }
    };

    // Projectiles, beams, damage text and VFX are never serialized. They are
    // tagged `DespawnOnExit(SceneState::Game)` and are already gone when this
    // system runs; this clears any stragglers defensively.
    for entity in &transients {
        commands.entity(entity).despawn();
    }

    let team = crate::content::team::Team::new(crate::content::team::DEFAULT_TEAM_ID);
    if let Err(e) = save.clone().restore(&mut commands, &registry, &config, team, None) {
        warn!("Failed to restore slot {slot}: {e}");
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
    *battle = BattleResult {
        started_at: Some(time.elapsed_secs() - save.battle.elapsed_secs),
        outcome: parse_outcome(&save.battle.outcome),
        duration_secs: save.battle.elapsed_secs,
        mission_name: save.battle.mission,
    };
    phase.set(BattlePhase::Active);
    notice.show(format!("Loaded slot {slot}"));
}

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SaveWorldEvent>()
            .add_message::<LoadWorldEvent>()
            .init_resource::<PendingLoad>()
            .add_systems(
                Update,
                (
                    handle_save_world_event.run_if(on_message::<SaveWorldEvent>),
                    handle_load_request.run_if(on_message::<LoadWorldEvent>),
                )
                    .in_set(SimulationSet::Present),
            )
            .add_systems(
                Update,
                begin_loaded_game
                    .run_if(in_state(SceneState::Loading))
                    .in_set(SimulationSet::Present),
            )
            .add_systems(OnEnter(SceneState::Game), spawn_loaded_world);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::save::document::parse_document;

    fn sample_save(slot: u8) -> WorldSave {
        WorldSave {
            version: SAVE_SCHEMA_VERSION,
            slot,
            saved_at_unix_ms: 1_700_000_000_000,
            battle: BattleSave {
                mission: "quick-battle".to_string(),
                elapsed_secs: 42.5,
                outcome: Some("Victory".to_string()),
            },
            camera: CameraSave {
                x: 12.0,
                y: -4.0,
                zoom: 3.0,
            },
            ships: Vec::new(),
            drops: Vec::new(),
            rng: RngSave { seed: 1234 },
        }
    }

    #[test]
    fn world_save_roundtrip_and_version() {
        let save = sample_save(2);
        let text = ron::ser::to_string_pretty(&save, ron::ser::PrettyConfig::default()).unwrap();
        let back: WorldSave = parse_document(&text).unwrap();
        assert_eq!(back, save);

        let bad = text.replace("version: 2", "version: 99");
        assert!(parse_document::<WorldSave>(&bad).is_err());
    }

    #[test]
    fn slots_are_one_to_five() {
        assert_eq!(SAVE_SLOTS, 5);
        for slot in 1..=5 {
            let path = slot_path(slot);
            assert!(path.ends_with(format!("slot_{slot}.ron")));
            assert!(path.to_string_lossy().contains("saves"));
        }
    }

    #[test]
    fn no_per_ship_paths_exist() {
        let save = sample_save(1);
        let text = ron::ser::to_string(&save).unwrap();
        assert!(!text.contains("ship_ids"));
        assert!(!text.contains("ships/"));
    }
}
