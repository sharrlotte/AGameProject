use bevy::prelude::*;

use crate::resource::{AppNotice, PauseState, SceneState, SettingState};
use crate::save::io::{LoadWorldEvent, SaveWorldEvent};
use crate::scene::layout::{BORDER_WIDTH, BUTTON_HEIGHT, BUTTON_WIDTH, CORNER_RADIUS, FONT_TITLE};
use crate::scene::slot_panel::{RefreshSlotPanel, SlotPanel, build_slot_panel};
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

fn handle_pause_input(
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

#[derive(Component)]
struct ResumeButton;

fn on_resume(
    click: On<PointerClick>,
    buttons: Query<&ResumeButton>,
    mut pause: ResMut<NextState<PauseState>>,
) {
    if buttons.contains(click.event().entity) {
        pause.set(PauseState::Playing);
    }
}

#[derive(Component)]
struct PauseSettingsButton;

fn on_settings(
    click: On<PointerClick>,
    buttons: Query<&PauseSettingsButton>,
    mut setting: ResMut<NextState<SettingState>>,
) {
    if buttons.contains(click.event().entity) {
        setting.set(SettingState::Open);
    }
}

#[derive(Component)]
struct SaveButton(pub u8);

fn on_save(
    click: On<PointerClick>,
    buttons: Query<&SaveButton>,
    mut saves: MessageWriter<SaveWorldEvent>,
    mut refresh: MessageWriter<RefreshSlotPanel>,
    mut notice: ResMut<AppNotice>,
) {
    let Ok(button) = buttons.get(click.event().entity) else { return; };
    saves.write(SaveWorldEvent { slot: button.0 });
    notice.show(format!("Saving slot {}…", button.0));
    refresh.write(RefreshSlotPanel);
}

#[derive(Component)]
struct LoadButton(pub u8);

fn on_load(
    click: On<PointerClick>,
    buttons: Query<&LoadButton>,
    mut pause: ResMut<NextState<PauseState>>,
    mut loads: MessageWriter<LoadWorldEvent>,
) {
    let Ok(button) = buttons.get(click.event().entity) else { return; };
    pause.set(PauseState::Playing);
    loads.write(LoadWorldEvent { slot: button.0 });
}

#[derive(Component)]
struct HomeButton;

fn on_home(
    click: On<PointerClick>,
    buttons: Query<&HomeButton>,
    mut pause: ResMut<NextState<PauseState>>,
    mut scene: ResMut<NextState<SceneState>>,
) {
    if buttons.contains(click.event().entity) {
        pause.set(PauseState::Playing);
        scene.set(SceneState::Home);
    }
}

#[derive(Component)]
struct ExitButton;

fn on_exit(
    click: On<PointerClick>,
    buttons: Query<&ExitButton>,
    mut exit: MessageWriter<AppExit>,
) {
    if buttons.contains(click.event().entity) {
        exit.write(AppExit::Success);
    }
}

fn setup_pause(mut commands: Commands, mut time: ResMut<Time<Virtual>>) {
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
                TextFont { font_size: FontSize::Px(40.0), ..default() },
                TextColor(Color::WHITE),
                Node { margin: UiRect::bottom(Val::Px(40.0)), ..default() },
            ));

            spawn_button(parent, button_node(), "Resume", 20.0, Color::WHITE,
                ResumeButton, on_resume, ButtonVisuals::dark());
            spawn_button(parent, button_node(), "Settings", 20.0, Color::WHITE,
                PauseSettingsButton, on_settings, ButtonVisuals::dark());
            spawn_button(parent, button_node(), "Save", 20.0, Color::WHITE,
                SaveButton(1), on_save, ButtonVisuals::green());
            spawn_button(parent, button_node(), "Load", 20.0, Color::WHITE,
                LoadButton(1), on_load, ButtonVisuals::blue());
            spawn_button(parent, button_node(), "Back to Home", 20.0, Color::WHITE,
                HomeButton, on_home, ButtonVisuals::dark());
            spawn_button(parent, button_node(), "Exit", 20.0, Color::WHITE,
                ExitButton, on_exit, ButtonVisuals::dark());

            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(4.0),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    SlotPanel { allow_save: true },
                ))
                .with_children(|panel| {
                    build_slot_panel(panel, true);
                });
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
