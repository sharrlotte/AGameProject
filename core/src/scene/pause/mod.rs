use bevy::prelude::*;

use crate::resource::{PauseState, SceneState, SettingState};
use crate::save::world::{EditorReturnTarget, PendingExitAction, SaveWorldEvent, WorldSession};
use crate::scene::layout::{BORDER_WIDTH, BUTTON_HEIGHT, BUTTON_WIDTH, CORNER_RADIUS, FONT_TITLE};
use crate::scene::widget::{ButtonVisuals, spawn_button};
use crate::simulation::SimulationSet;

pub struct PauseScenePlugin;

impl Plugin for PauseScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(PauseState::Paused), setup_pause)
            .add_systems(
                Update,
                handle_pause_input
                    .run_if(in_state(SceneState::Game).or_else(in_state(SceneState::ShipEditor)))
                    .in_set(SimulationSet::Input),
            )
            .add_systems(OnExit(PauseState::Paused), cleanup_pause);
    }
}

#[derive(Component)]
struct ResumeButton;

#[derive(Component)]
struct PauseSettingsButton;

#[derive(Component)]
struct PauseShipEditorButton;

#[derive(Component)]
struct HomeButton;

pub fn handle_pause_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<NextState<PauseState>>,
    state: Res<State<PauseState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        let next_state: PauseState = match state.get() {
            PauseState::Paused => PauseState::Playing,
            PauseState::Playing => PauseState::Paused,
        };
        game_state.set(next_state);
    }
}

fn on_resume(
    click: On<PointerClick>,
    buttons: Query<&ResumeButton>,
    mut pause: ResMut<NextState<PauseState>>,
) {
    if buttons.contains(click.event().entity) {
        pause.set(PauseState::Playing);
    }
}

fn on_settings(
    click: On<PointerClick>,
    buttons: Query<&PauseSettingsButton>,
    mut setting: ResMut<NextState<SettingState>>,
) {
    if buttons.contains(click.event().entity) {
        setting.set(SettingState::Open);
    }
}

fn on_ship_editor(
    click: On<PointerClick>,
    buttons: Query<&PauseShipEditorButton>,
    session: Res<WorldSession>,
    mut target: ResMut<EditorReturnTarget>,
    mut pause: ResMut<NextState<PauseState>>,
    mut scene: ResMut<NextState<SceneState>>,
    mut saves: MessageWriter<SaveWorldEvent>,
) {
    if buttons.contains(click.event().entity) {
        if matches!(*session, WorldSession::SaveSlot { .. }) {
            saves.write(SaveWorldEvent);
        }
        *target = EditorReturnTarget::Game;
        pause.set(PauseState::Playing);
        scene.set(SceneState::ShipEditor);
    }
}

fn on_home(
    click: On<PointerClick>,
    buttons: Query<&HomeButton>,
    scene_state: Res<State<SceneState>>,
    session: Res<WorldSession>,
    mut pause: ResMut<NextState<PauseState>>,
    mut scene: ResMut<NextState<SceneState>>,
    mut saves: MessageWriter<SaveWorldEvent>,
    mut pending_exit: ResMut<PendingExitAction>,
) {
    if !buttons.contains(click.event().entity) {
        return;
    }
    pause.set(PauseState::Playing);
    if *scene_state.get() == SceneState::Game
        && matches!(*session, WorldSession::SaveSlot { .. })
    {
        saves.write(SaveWorldEvent);
        *pending_exit = PendingExitAction::Home;
    } else {
        scene.set(SceneState::Home);
    }
}

fn setup_pause(mut commands: Commands, mut time: ResMut<Time<Virtual>>, session: Res<WorldSession>) {
    time.pause();

    commands
        .spawn((
            Transform::from_xyz(0.0, 0.0, 100.0),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(FONT_TITLE),
                ..default()
            },
            DespawnOnExit(PauseState::Paused),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Paused"),
                TextFont {
                    font_size: FontSize::Px(40.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ));

            spawn_button(
                parent,
                button_node(),
                "Resume",
                20.0,
                Color::WHITE,
                ResumeButton,
                on_resume,
                ButtonVisuals::dark(),
            );
            spawn_button(
                parent,
                button_node(),
                "Settings",
                20.0,
                Color::WHITE,
                PauseSettingsButton,
                on_settings,
                ButtonVisuals::dark(),
            );
            if matches!(*session, WorldSession::SaveSlot { .. }) {
                spawn_button(
                    parent,
                    button_node(),
                    "Ship Editor",
                    20.0,
                    Color::WHITE,
                    PauseShipEditorButton,
                    on_ship_editor,
                    ButtonVisuals::blue(),
                );
            }
            spawn_button(
                parent,
                button_node(),
                "Back to Home",
                20.0,
                Color::WHITE,
                HomeButton,
                on_home,
                ButtonVisuals::dark(),
            );
        });
}

fn button_node() -> Node {
    Node {
        width: Val::Px(BUTTON_WIDTH),
        height: Val::Px(BUTTON_HEIGHT),
        border: UiRect::all(Val::Px(BORDER_WIDTH)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        padding: UiRect::all(Val::Px(CORNER_RADIUS)),
        border_radius: BorderRadius::all(Val::Px(CORNER_RADIUS)),
        ..default()
    }
}

fn cleanup_pause(mut time: ResMut<Time<Virtual>>) {
    time.unpause();
}
