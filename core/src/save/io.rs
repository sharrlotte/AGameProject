use bevy::prelude::*;
use directories::ProjectDirs;
use std::{
    fs, io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::config::GameConfigValue;
use crate::content::block::BlockRegistry;
use crate::content::team::{DEFAULT_TEAM_ID, Team};
use crate::mission::{BattleOutcome, BattlePhase, BattleResult};
use crate::resource::{
    AppNotice, BattleMode, CameraTarget, ControlledShip, GameRng, PendingLoad, SceneState,
};
use crate::save::document::{DocumentError, read_document, write_document};
use crate::save::schema::*;
use crate::save::snapshot::{CaptureContext, CaptureSnapshot, RestoreSnapshot};
use crate::simulation::SimulationSet;

#[derive(Message)]
pub struct SaveWorldEvent;

#[derive(Message)]
pub struct LoadWorldEvent;

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PendingExitAction {
    #[default]
    None,
    Home,
}

#[derive(Resource, Default)]
pub struct SaveResult {
    pub message: Option<String>,
}

pub fn data_dir() -> PathBuf {
    ProjectDirs::from("com", "somerandomdev", "agameproject")
        .map(|dirs| dirs.data_dir().to_path_buf())
        .unwrap_or_else(|| {
            error!("Failed to resolve project directory, falling back to local ./agameproject");
            PathBuf::from("./agameproject")
        })
}

pub fn battle_save_path() -> PathBuf {
    battle_save_path_in(&data_dir())
}

pub fn campaign_save_path() -> PathBuf {
    campaign_save_path_in(&data_dir())
}

pub fn battle_save_path_in(base: &Path) -> PathBuf {
    base.join("saves").join("battle.ron")
}

pub fn campaign_save_path_in(base: &Path) -> PathBuf {
    base.join("saves").join("campaign.ron")
}

pub fn has_battle_save() -> bool {
    battle_save_path().is_file()
}

pub fn load_campaign() -> CampaignSave {
    load_campaign_in(&data_dir())
}

pub fn load_campaign_in(base: &Path) -> CampaignSave {
    read_document::<CampaignSave>(&campaign_save_path_in(base)).unwrap_or_default()
}

pub fn save_campaign(campaign: &CampaignSave) -> Result<(), DocumentError> {
    write_document(&campaign_save_path(), campaign)
}

fn remove_with_backup(path: &Path) {
    if let Err(err) = fs::remove_file(path)
        && err.kind() != io::ErrorKind::NotFound
    {
        warn!("Failed to remove {}: {err}", path.display());
    }
    let bak = path.with_extension("ron.bak");
    if let Err(err) = fs::remove_file(&bak)
        && err.kind() != io::ErrorKind::NotFound
    {
        warn!("Failed to remove {}: {err}", bak.display());
    }
}

pub fn clear_battle_save() {
    clear_battle_save_in(&data_dir());
}

pub fn clear_battle_save_in(base: &Path) {
    remove_with_backup(&battle_save_path_in(base));
}

pub fn reset_campaign_data() {
    reset_campaign_data_in(&data_dir());
}

pub fn reset_campaign_data_in(base: &Path) {
    remove_with_backup(&battle_save_path_in(base));
    remove_with_backup(&campaign_save_path_in(base));
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
    let save = WorldSave::capture(&ctx, Entity::PLACEHOLDER);

    let message = match write_document(&battle_save_path(), &save) {
        Ok(()) => None,
        Err(e) => Some(format!("Save failed: {e}")),
    };
    if let Some(message) = message {
        commands.insert_resource(SaveResult {
            message: Some(message),
        });
    }
}

fn flush_pending_exit_action(
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

    // Deferred so other OnEnter(Game) systems still observe the pending flag
    // this frame and skip mission setup.
    commands.insert_resource(PendingLoad::default());
    commands.insert_resource(CameraTarget::default());
    commands.insert_resource(ControlledShip::default());

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

    let team = Team::new(DEFAULT_TEAM_ID);
    if let Err(e) = save
        .clone()
        .restore(&mut commands, &registry, &config, team, None)
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
    *battle = BattleResult {
        started_at: Some(time.elapsed_secs() - save.battle.elapsed_secs),
        outcome: parse_outcome(&save.battle.outcome),
        duration_secs: save.battle.elapsed_secs,
        mission_name: save.battle.mission,
    };
    phase.set(BattlePhase::Active);
}

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SaveWorldEvent>()
            .add_message::<LoadWorldEvent>()
            .init_resource::<SaveResult>()
            .init_resource::<PendingLoad>()
            .init_resource::<BattleMode>()
            .init_resource::<PendingExitAction>()
            .add_systems(
                Update,
                (
                    handle_save_world_event
                        .run_if(on_message::<SaveWorldEvent>)
                        .before(flush_pending_exit_action),
                    flush_pending_exit_action,
                    handle_load_request.run_if(on_message::<LoadWorldEvent>),
                    report_save_result,
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
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!("agameproject_{label}_{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn sample_save() -> WorldSave {
        WorldSave {
            version: SAVE_SCHEMA_VERSION,
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
        let save = sample_save();
        let text = ron::ser::to_string_pretty(&save, ron::ser::PrettyConfig::default()).unwrap();
        let back: WorldSave = parse_document(&text).unwrap();
        assert_eq!(back, save);

        let bad = text.replace(&format!("version: {SAVE_SCHEMA_VERSION}"), "version: 99");
        assert!(parse_document::<WorldSave>(&bad).is_err());
    }

    #[test]
    fn single_battle_and_campaign_paths() {
        let battle = battle_save_path();
        assert!(battle.ends_with("battle.ron"));
        assert!(battle.to_string_lossy().contains("saves"));

        let campaign = campaign_save_path();
        assert!(campaign.ends_with("campaign.ron"));
        assert!(campaign.to_string_lossy().contains("saves"));
    }

    #[test]
    fn campaign_roundtrip_and_reset_preserves_designs() {
        let root = temp_dir("campaign_reset");
        let designs_dir = root.join("designs");
        fs::create_dir_all(&designs_dir).unwrap();
        let design_file = designs_dir.join("my_ship.ron");
        fs::write(&design_file, "(version: 1)").unwrap();

        let campaign = CampaignSave {
            version: CAMPAIGN_SCHEMA_VERSION,
            current_mission: "quick-battle".to_string(),
            completed_missions: vec!["quick-battle".to_string()],
        };
        write_document(&campaign_save_path_in(&root), &campaign).unwrap();
        write_document(&battle_save_path_in(&root), &sample_save()).unwrap();

        assert_eq!(load_campaign_in(&root), campaign);

        reset_campaign_data_in(&root);
        assert!(!battle_save_path_in(&root).exists());
        assert!(!campaign_save_path_in(&root).exists());
        assert!(design_file.exists());

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn no_per_ship_paths_exist() {
        let save = sample_save();
        let text = ron::ser::to_string(&save).unwrap();
        assert!(!text.contains("ship_ids"));
        assert!(!text.contains("ships/"));
    }
}
