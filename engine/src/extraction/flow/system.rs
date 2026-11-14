use bevy::prelude::*;
use crate::common::{GameScene, GameState, RunEntity};
use crate::message::messages::{
    DeployRequestedMessage, EnemyKilledMessage, ExtractionCompletedMessage, OpenUpgradesRequestedMessage,
    PlayerDiedMessage, ScrapCollectedMessage,
};
use crate::message::resources::{LastRunOutcome, RunStats, ScrapBank};
use crate::config::ExtractionConfig;

pub fn reset_run_stats(mut stats: ResMut<RunStats>, extraction_cfg: Res<ExtractionConfig>) {
    *stats = RunStats {
        extraction_hold_secs: extraction_cfg.hold_duration_secs,
        ..default()
    };
}

pub fn cleanup_run_entities(mut commands: Commands, run_entities: Query<Entity, With<RunEntity>>) {
    for entity in &run_entities {
        commands.entity(entity).despawn();
    }
}

pub fn tick_run_timer(time: Res<Time>, mut stats: ResMut<RunStats>) {
    stats.elapsed_secs += time.delta_secs();
}

pub fn accumulate_kills_on_event(
    mut killed_reader: MessageReader<EnemyKilledMessage>,
    mut stats: ResMut<RunStats>,
) {
    for _ in killed_reader.read() {
        stats.kills += 1;
    }
}

pub fn accumulate_scrap_on_event(
    mut scrap_reader: MessageReader<ScrapCollectedMessage>,
    mut stats: ResMut<RunStats>,
) {
    for msg in scrap_reader.read() {
        stats.run_scrap += msg.amount;
    }
}

pub fn handle_player_died_event(
    mut died_reader: MessageReader<PlayerDiedMessage>,
    mut stats: ResMut<RunStats>,
    mut outcome: ResMut<LastRunOutcome>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if died_reader.read().next().is_some() {
        *outcome = LastRunOutcome::Died {
            scrap_lost: stats.run_scrap,
            kills: stats.kills,
            elapsed_secs: stats.elapsed_secs,
        };
        stats.run_scrap = 0;
        next_state.set(GameState::GameOver);
    }
}

pub fn handle_extraction_completed_event(
    mut extracted_reader: MessageReader<ExtractionCompletedMessage>,
    stats: Res<RunStats>,
    mut bank: ResMut<ScrapBank>,
    mut outcome: ResMut<LastRunOutcome>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if extracted_reader.read().next().is_some() {
        bank.banked += stats.run_scrap;
        *outcome = LastRunOutcome::Extracted {
            scrap_banked: stats.run_scrap,
            kills: stats.kills,
            elapsed_secs: stats.elapsed_secs,
        };
        next_state.set(GameState::Upgrades);
    }
}

pub fn handle_navigation_requests(
    mut deploy_reader: MessageReader<DeployRequestedMessage>,
    mut upgrades_reader: MessageReader<OpenUpgradesRequestedMessage>,
    mut next_state: ResMut<NextState<GameState>>,
    mut next_scene: ResMut<NextState<GameScene>>,
) {
    if deploy_reader.read().next().is_some() {
        next_scene.set(GameScene::Arena);
        next_state.set(GameState::Playing);
    } else if upgrades_reader.read().next().is_some() {
        next_state.set(GameState::Upgrades);
    }
}

