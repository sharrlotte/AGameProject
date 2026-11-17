pub mod def;
mod state;

pub use def::MissionDef;
pub use state::{BattleOutcome, BattlePhase, BattleRestart, BattleResult};

use bevy::asset::LoadState;
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;

use crate::config::GameConfigValue;
use crate::content::block::BlockRegistry;
use crate::content::ship::Ship;
use crate::content::team::Team;
use crate::config::ConfigState;
use crate::resource::{AppNotice, DeployPayload, PendingLoad, SceneState};

use crate::mission::state::spawn_mission;
use crate::simulation::SimulationSet;

#[derive(Resource)]
struct MissionAsset(Handle<MissionDef>);

pub struct MissionPlugin;

impl Plugin for MissionPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<BattlePhase>()
            .init_asset::<MissionDef>()
            .init_resource::<BattleResult>()
            .init_resource::<BattleRestart>()
            .register_asset_loader(crate::config::loader::RonAssetLoader::<MissionDef>::new(
                &["ron"],
            ))
            .add_systems(
                OnEnter(SceneState::Game),
                setup_battle.in_set(SimulationSet::Input),
            )
            .add_systems(
                OnEnter(BattlePhase::Ended),
                spawn_end_overlay.in_set(SimulationSet::Input),
            )
            .add_systems(
                Update,
                poll_mission_loaded
                    .run_if(in_state(BattlePhase::Loading))
                    .run_if(in_state(ConfigState::Ready))
                    .in_set(SimulationSet::Present),
            )
            .add_systems(
                Update,
                handle_end_buttons
                    .run_if(in_state(BattlePhase::Ended))
                    .in_set(SimulationSet::Input),
            )
            .add_systems(
                OnEnter(SceneState::Home),
                resume_battle_on_restart.in_set(SimulationSet::Input),
            )
            .add_systems(
                FixedUpdate,
                check_battle_outcome
                    .run_if(in_state(BattlePhase::Active))
                    .in_set(SimulationSet::Damage),
            );
    }
}

fn setup_battle(
    mut commands: Commands,
    assets: Res<AssetServer>,
    pending: Res<PendingLoad>,
    mut phase: ResMut<NextState<BattlePhase>>,
) {
    // A snapshot load takes over this Game entry; mission setup must not
    // overwrite it with a fresh battle.
    if pending.is_pending() {
        return;
    }
    commands.insert_resource(MissionAsset(assets.load("config/mission/quick-battle.ron")));
    commands.insert_resource(BattleResult::default());
    phase.set(BattlePhase::Loading);
}

fn poll_mission_loaded(
    mut commands: Commands,
    assets: Res<Assets<MissionDef>>,
    server: Res<AssetServer>,
    handle: Option<Res<MissionAsset>>,
    registry: Res<BlockRegistry>,
    config: Res<GameConfigValue>,
    state: Res<DeployPayload>,
    mut result: ResMut<BattleResult>,
    mut phase: ResMut<NextState<BattlePhase>>,
    mut notice: ResMut<AppNotice>,
    mut scene_state: ResMut<NextState<SceneState>>,
    time: Res<Time>,
) {
    let Some(handle) = handle else {
        return;
    };

    if matches!(
        server.get_load_state(&handle.0),
        Some(LoadState::Failed(_))
    ) {
        warn!("config/mission/quick-battle.ron failed to load");
        notice.show("Mission failed to load; returning home.");
        scene_state.set(SceneState::Home);
        return;
    }

    let Some(mission) = assets.get(&handle.0).cloned() else {
        return;
    };

    spawn_mission(&mut commands, &registry, &config, &mission, &state);

    result.started_at = Some(time.elapsed_secs());
    result.outcome = None;
    result.duration_secs = 0.0;

    phase.set(BattlePhase::Active);
}

fn check_battle_outcome(
    ships: Query<&Team, With<Ship>>,
    mut result: ResMut<BattleResult>,
    mut phase: ResMut<NextState<BattlePhase>>,
    time: Res<Time>,
) {
    let mut player_ships = 0usize;
    let mut enemy_ships = 0usize;

    for team in &ships {
        if team.is_player() {
            player_ships += 1;
        } else {
            enemy_ships += 1;
        }
    }

    let Some(started_at) = result.started_at else {
        return;
    };

    if player_ships == 0 {
        result.outcome = Some(BattleOutcome::Defeat);
        result.duration_secs = time.elapsed_secs() - started_at;
        phase.set(BattlePhase::Ended);
    } else if enemy_ships == 0 {
        result.outcome = Some(BattleOutcome::Victory);
        result.duration_secs = time.elapsed_secs() - started_at;
        phase.set(BattlePhase::Ended);
    }
}

#[derive(Component)]
enum BattleEndButton {
    Retry,
    Home,
}

fn spawn_end_overlay(mut commands: Commands, result: Res<BattleResult>) {
    let (title, color) = match result.outcome {
        Some(BattleOutcome::Victory) => ("VICTORY", Color::srgb(0.4, 1.0, 0.4)),
        Some(BattleOutcome::Defeat) => ("DEFEAT", Color::srgb(1.0, 0.4, 0.4)),
        None => ("BATTLE ENDED", Color::WHITE),
    };

    let time_text = format!("Time: {:.1}s", result.duration_secs);

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            GlobalZIndex(100),
            DespawnOnExit(SceneState::Game),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(title),
                TextFont {
                    font_size: FontSize::Px(80.0),
                    ..default()
                },
                TextColor(color),
            ));

            parent.spawn((
                Text::new(time_text),
                TextFont {
                    font_size: FontSize::Px(24.0),
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));

            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(20.0),
                    ..default()
                })
                .with_children(|row| {
                    for (label, button) in [
                        ("Retry", BattleEndButton::Retry),
                        ("Home", BattleEndButton::Home),
                    ] {
                        row.spawn((
                            Node {
                                width: Val::Px(180.0),
                                height: Val::Px(50.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                            button,
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new(label),
                                TextFont {
                                    font_size: FontSize::Px(20.0),
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                    }
                });
        });
}

fn handle_end_buttons(
    interactions: Query<
        (&PickingInteraction, &BattleEndButton),
        Changed<PickingInteraction>,
    >,
    mut next_state: ResMut<NextState<SceneState>>,
    mut restart: ResMut<BattleRestart>,
) {
    for (interaction, button) in &interactions {
        if *interaction != PickingInteraction::Pressed {
            continue;
        }

        match button {
            BattleEndButton::Retry => {
                *restart = BattleRestart(true);
                next_state.set(SceneState::Home);
            }
            BattleEndButton::Home => {
                next_state.set(SceneState::Home);
            }
        }
    }
}

fn resume_battle_on_restart(
    mut restart: ResMut<BattleRestart>,
    mut next_state: ResMut<NextState<SceneState>>,
) {
    if restart.0 {
        *restart = BattleRestart(false);
        next_state.set(SceneState::Game);
    }
}
