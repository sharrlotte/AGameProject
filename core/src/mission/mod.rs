pub mod def;

mod state;


use std::collections::HashSet;


pub use def::MissionDef;

pub use state::{BattleOutcome, BattlePhase, BattleRestart, BattleResult};


use bevy::asset::LoadState;

use bevy::prelude::*;


use crate::config::ConfigState;

use crate::config::GameConfigValue;

use crate::config::loader::RonAssetLoader;

use crate::content::block::BlockRegistry;

use crate::content::death::ShipDestroyed;

use crate::content::body::mutation::apply_body_mutations;

use crate::content::ship::Ship;

use crate::content::team::Team;

use crate::resource::{AppNotice, BattleMode, DeployPayload, PauseState, PendingLoad, SceneState};

use crate::save::battle::{BattleSaveState, SaveWorldEvent, clear_battle_save};

use crate::save::campaign::CampaignSave;

use crate::save::design::ShipCatalog;


use crate::mission::state::spawn_mission;

use crate::scene::widget::{ButtonVisuals, spawn_button};

use crate::simulation::SimulationSet;


#[derive(Resource)]
struct MissionAsset(Handle<MissionDef>);


#[derive(Resource)]
struct AutoSaveTimer(Timer);


pub struct MissionPlugin;


impl Plugin for MissionPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<BattlePhase>()
            .init_asset::<MissionDef>()
            .init_resource::<BattleResult>()
            .init_resource::<BattleRestart>()
            .register_asset_loader(RonAssetLoader::<MissionDef>::new(&["ron"]))
            .add_systems(
                OnEnter(SceneState::Game),
                (setup_battle, reset_auto_save_timer).in_set(SimulationSet::Input),
            )
            .add_systems(
                OnEnter(BattlePhase::Ended),
                (spawn_end_overlay, record_campaign_outcome).in_set(SimulationSet::Input),
            )
            .add_systems(
                Update,
                poll_mission_loaded
                    .run_if(in_state(SceneState::Game))
                    .run_if(in_state(BattlePhase::Loading))
                    .run_if(in_state(ConfigState::Ready))
                    .in_set(SimulationSet::Present),
            )
            .add_systems(
                Update,
                tick_auto_save
                    .run_if(in_state(SceneState::Game))
                    .run_if(in_state(BattlePhase::Active))
                    .run_if(in_state(PauseState::Playing))
                    .in_set(SimulationSet::Present),
            )
            .add_systems(
                OnEnter(SceneState::Home),
                resume_battle_on_restart.in_set(SimulationSet::Input),
            )
            .add_systems(
                OnExit(SceneState::Game),
                reset_battle.in_set(SimulationSet::Input),
            )
            .add_systems(
                FixedPostUpdate,
                check_battle_outcome
                    .after(apply_body_mutations)
                    .run_if(in_state(SceneState::Game))
                    .run_if(in_state(BattlePhase::Active))
                    .in_set(SimulationSet::Death),
            );
    }
}


#[derive(Component)]
struct RetryButton;


#[derive(Component)]
struct EndHomeButton;


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
    commands.insert_resource(BattleResult {
        mission_name: "quick-battle".to_string(),
        ..default()
    });
    phase.set(BattlePhase::Loading);
}


fn reset_auto_save_timer(mut commands: Commands, config: Res<GameConfigValue>) {
    let interval = config.0.auto_save_interval_secs.max(1.0);
    commands.insert_resource(AutoSaveTimer(Timer::from_seconds(
        interval,
        TimerMode::Repeating,
    )));
}


fn tick_auto_save(
    time: Res<Time>,
    mode: Res<BattleMode>,
    mut timer: Option<ResMut<AutoSaveTimer>>,
    mut save_events: MessageWriter<SaveWorldEvent>,
) {
    if *mode != BattleMode::Campaign {
        return;
    }
    let Some(ref mut timer) = timer else {
        return;
    };
    if timer.0.tick(time.delta()).just_finished() {
        save_events.write(SaveWorldEvent);
    }
}


fn record_campaign_outcome(
    mode: Res<BattleMode>,
    result: Res<BattleResult>,
    mut campaign: ResMut<CampaignSave>,
    mut battle_save: ResMut<BattleSaveState>,
) {
    if *mode != BattleMode::Campaign {
        return;
    }
    clear_battle_save();
    battle_save.has_save = false;
    if result.outcome == Some(BattleOutcome::Victory)
        && let Err(err) = campaign.record_victory(&result.mission_name)
    {
        warn!("Failed to save campaign progress: {err}");
    }
}


fn poll_mission_loaded(
    mut commands: Commands,
    assets: Res<Assets<MissionDef>>,
    server: Res<AssetServer>,
    handle: Option<Res<MissionAsset>>,
    registry: Res<BlockRegistry>,
    config: Res<GameConfigValue>,
    state: Res<DeployPayload>,
    catalog: Res<ShipCatalog>,
    mut result: ResMut<BattleResult>,
    mut phase: ResMut<NextState<BattlePhase>>,
    mut notice: ResMut<AppNotice>,
    mut scene_state: ResMut<NextState<SceneState>>,
    time: Res<Time>,
) {
    let Some(handle) = handle else {
        return;
    };

    if matches!(server.get_load_state(&handle.0), Some(LoadState::Failed(_))) {
        warn!("config/mission/quick-battle.ron failed to load");
        notice.show("Mission failed to load; returning home.");
        scene_state.set(SceneState::Home);
        return;
    }

    let Some(mission) = assets.get(&handle.0).cloned() else {
        return;
    };

    spawn_mission(&mut commands, &registry, &config, &mission, &state, &catalog);

    result.started_at = Some(time.elapsed_secs());
    result.outcome = None;
    result.duration_secs = 0.0;

    phase.set(BattlePhase::Active);
}


fn reset_battle(
    mut result: ResMut<BattleResult>,
    mut phase: ResMut<NextState<BattlePhase>>,
) {
    *result = BattleResult::default();
    phase.set(BattlePhase::Idle);
}


fn check_battle_outcome(
    mut events: MessageReader<ShipDestroyed>,
    ships: Query<(Entity, &Team), With<Ship>>,
    mut result: ResMut<BattleResult>,
    mut phase: ResMut<NextState<BattlePhase>>,
    time: Res<Time>,
) {
    let mut destroyed = HashSet::new();
    let mut player_destroyed = false;
    let mut enemy_destroyed = false;
    for event in events.read() {
        destroyed.insert(event.ship);
        if event.team.is_player() {
            player_destroyed = true;
        } else {
            enemy_destroyed = true;
        }
    }
    if destroyed.is_empty() {
        return;
    }

    let Some(started_at) = result.started_at else {
        return;
    };

    let mut player_ships = 0usize;
    let mut enemy_ships = 0usize;
    for (entity, team) in &ships {
        if destroyed.contains(&entity) {
            continue;
        }
        if team.is_player() {
            player_ships += 1;
        } else {
            enemy_ships += 1;
        }
    }

    if player_destroyed && player_ships == 0 {
        result.outcome = Some(BattleOutcome::Defeat);
        result.duration_secs = time.elapsed_secs() - started_at;
        phase.set(BattlePhase::Ended);
    } else if enemy_destroyed && enemy_ships == 0 {
        result.outcome = Some(BattleOutcome::Victory);
        result.duration_secs = time.elapsed_secs() - started_at;
        phase.set(BattlePhase::Ended);
    }
}


fn on_retry(
    click: On<PointerClick>,
    buttons: Query<&RetryButton>,
    mut restart: ResMut<BattleRestart>,
    mut scene: ResMut<NextState<SceneState>>,
) {
    if buttons.contains(click.event().entity) {
        *restart = BattleRestart(true);
        scene.set(SceneState::Home);
    }
}


fn on_end_home(
    click: On<PointerClick>,
    buttons: Query<&EndHomeButton>,
    mut scene: ResMut<NextState<SceneState>>,
) {
    if buttons.contains(click.event().entity) {
        scene.set(SceneState::Home);
    }
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
                    spawn_button(
                        row,
                        end_button_node(),
                        "Retry",
                        20.0,
                        Color::WHITE,
                        RetryButton,
                        on_retry,
                        ButtonVisuals::dark(),
                    );
                    spawn_button(
                        row,
                        end_button_node(),
                        "Home",
                        20.0,
                        Color::WHITE,
                        EndHomeButton,
                        on_end_home,
                        ButtonVisuals::dark(),
                    );
                });
        });
}


fn end_button_node() -> Node {
    Node {
        width: Val::Px(180.0),
        height: Val::Px(50.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
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


#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::death::ShipDestroyed;
    use bevy::state::app::StatesPlugin;

    #[derive(Resource, Default)]
    struct EndedSpawned(bool);

    fn setup() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(StatesPlugin)
            .init_state::<SceneState>()
            .init_state::<BattlePhase>()
            .init_resource::<BattleResult>()
            .init_resource::<EndedSpawned>()
            .add_message::<ShipDestroyed>()
            .configure_sets(FixedPostUpdate, SimulationSet::Death)
            .add_systems(OnExit(SceneState::Game), reset_battle)
            .add_systems(
                OnEnter(BattlePhase::Ended),
                |mut flag: ResMut<EndedSpawned>| flag.0 = true,
            )
            .add_systems(
                FixedPostUpdate,
                check_battle_outcome
                    .after(apply_body_mutations)
                    .run_if(in_state(SceneState::Game))
                    .run_if(in_state(BattlePhase::Active))
                    .in_set(SimulationSet::Death),
            );
        app
    }

    fn enter_battle(app: &mut App) {
        app.world_mut()
            .resource_mut::<NextState<SceneState>>()
            .set(SceneState::Game);
        app.world_mut()
            .resource_mut::<NextState<BattlePhase>>()
            .set(BattlePhase::Active);
        app.update();
        app.world_mut().resource_mut::<BattleResult>().started_at = Some(0.0);
    }

    fn spawn_ship(app: &mut App, team: Team) -> Entity {
        app.world_mut().spawn((Ship::default(), team)).id()
    }

    fn emit(app: &mut App, ship: Entity, team: Team) {
        app.world_mut()
            .resource_mut::<Messages<ShipDestroyed>>()
            .write(ShipDestroyed { ship, team });
    }

    fn tick(app: &mut App) {
        app.world_mut().run_schedule(FixedPostUpdate);
    }

    #[test]
    fn last_enemy_core_yields_victory() {
        let mut app = setup();
        enter_battle(&mut app);
        spawn_ship(&mut app, Team::new(0));
        let enemy = spawn_ship(&mut app, Team::new(1));
        emit(&mut app, enemy, Team::new(1));
        tick(&mut app);
        assert_eq!(
            app.world().resource::<BattleResult>().outcome,
            Some(BattleOutcome::Victory)
        );
    }

    #[test]
    fn last_player_core_yields_defeat() {
        let mut app = setup();
        enter_battle(&mut app);
        let player = spawn_ship(&mut app, Team::new(0));
        spawn_ship(&mut app, Team::new(1));
        emit(&mut app, player, Team::new(0));
        tick(&mut app);
        assert_eq!(
            app.world().resource::<BattleResult>().outcome,
            Some(BattleOutcome::Defeat)
        );
    }

    #[test]
    fn battle_continues_when_other_ships_remain() {
        let mut app = setup();
        enter_battle(&mut app);
        let player = spawn_ship(&mut app, Team::new(0));
        spawn_ship(&mut app, Team::new(0));
        spawn_ship(&mut app, Team::new(1));
        emit(&mut app, player, Team::new(0));
        tick(&mut app);
        assert_eq!(app.world().resource::<BattleResult>().outcome, None);
    }

    #[test]
    fn no_destroyed_message_does_not_end_battle() {
        let mut app = setup();
        enter_battle(&mut app);
        spawn_ship(&mut app, Team::new(0));
        tick(&mut app);
        assert_eq!(app.world().resource::<BattleResult>().outcome, None);
    }

    #[test]
    fn returning_home_resets_battle_without_overlay() {
        let mut app = setup();
        enter_battle(&mut app);
        app.world_mut().resource_mut::<BattleResult>().outcome = Some(BattleOutcome::Victory);
        app.world_mut()
            .resource_mut::<NextState<SceneState>>()
            .set(SceneState::Home);
        app.update();
        app.update();
        tick(&mut app);
        assert_eq!(app.world().resource::<BattleResult>().outcome, None);
        assert_eq!(
            *app.world().resource::<State<BattlePhase>>().get(),
            BattlePhase::Idle
        );
        assert!(!app.world().resource::<EndedSpawned>().0);
    }

    #[test]
    fn outcome_event_outside_game_is_ignored() {
        let mut app = setup();
        app.world_mut()
            .resource_mut::<NextState<BattlePhase>>()
            .set(BattlePhase::Active);
        app.update();
        app.world_mut().resource_mut::<BattleResult>().started_at = Some(0.0);
        let enemy = spawn_ship(&mut app, Team::new(1));
        emit(&mut app, enemy, Team::new(1));
        tick(&mut app);
        assert_eq!(app.world().resource::<BattleResult>().outcome, None);
    }
}
