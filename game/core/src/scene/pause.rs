use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;

use crate::{
    resource::{PauseState, SceneState, SettingState},
    save::io::{LoadWorldEvent, SaveWorldEvent},
    scene::layout::{BORDER_WIDTH, BUTTON_HEIGHT, BUTTON_WIDTH, CORNER_RADIUS, FONT_TITLE},
    scene::slot_panel::{RefreshSlotPanel, SlotPanel, build_slot_panel},
    simulation::SimulationSet,
};

pub struct PauseScenePlugin;

impl Plugin for PauseScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(PauseState::Paused), setup_pause)
            .add_systems(
                Update,
                handle_pause_buttons
                    .run_if(in_state(PauseState::Paused))
                    .in_set(SimulationSet::Input),
            )
            .add_systems(OnExit(PauseState::Paused), cleanup_pause)
            .add_systems(
                Update,
                handle_pause_input
                    .run_if(in_state(SceneState::Game).or_else(in_state(SceneState::ShipEditor)))
                    .in_set(SimulationSet::Input),
            );
    }
}

#[derive(Component)]
enum PauseButton {
    Resume,
    Settings,
    Home,
    Save,
    Load,
    Exit,
}

#[derive(Component)]
struct PauseScreen;

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
            PauseScreen,
            DespawnOnExit(PauseState::Paused),
        ))
        .with_children(|parent| {
            // Title
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

            // Resume Button
            parent
                .spawn((
                    Node {
                        width: Val::Px(BUTTON_WIDTH),
                        height: Val::Px(BUTTON_HEIGHT),
                        border: UiRect::all(Val::Px(BORDER_WIDTH)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(CORNER_RADIUS)),
                        border_radius: BorderRadius::all(Val::Px(CORNER_RADIUS)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    PauseButton::Resume,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Resume"),
                        TextFont {
                            font_size: FontSize::Px(20.0),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // Settings Button
            parent
                .spawn((
                    Node {
                        width: Val::Px(BUTTON_WIDTH),
                        height: Val::Px(BUTTON_HEIGHT),
                        border: UiRect::all(Val::Px(BORDER_WIDTH)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(CORNER_RADIUS)),
                        border_radius: BorderRadius::all(Val::Px(CORNER_RADIUS)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    PauseButton::Settings,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Settings"),
                        TextFont {
                            font_size: FontSize::Px(20.0),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // Save Button
            parent
                .spawn((
                    Node {
                        width: Val::Px(BUTTON_WIDTH),
                        height: Val::Px(BUTTON_HEIGHT),
                        border: UiRect::all(Val::Px(BORDER_WIDTH)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(CORNER_RADIUS)),
                        border_radius: BorderRadius::all(Val::Px(CORNER_RADIUS)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    PauseButton::Save,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Save"),
                        TextFont {
                            font_size: FontSize::Px(20.0),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
            parent
                .spawn((
                    Node {
                        width: Val::Px(BUTTON_WIDTH),
                        height: Val::Px(BUTTON_HEIGHT),
                        border: UiRect::all(Val::Px(BORDER_WIDTH)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(CORNER_RADIUS)),
                        border_radius: BorderRadius::all(Val::Px(CORNER_RADIUS)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    PauseButton::Load,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Load"),
                        TextFont {
                            font_size: FontSize::Px(20.0),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // Home Button
            parent
                .spawn((
                    Node {
                        width: Val::Px(BUTTON_WIDTH),
                        height: Val::Px(BUTTON_HEIGHT),
                        border: UiRect::all(Val::Px(BORDER_WIDTH)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(CORNER_RADIUS)),
                        border_radius: BorderRadius::all(Val::Px(CORNER_RADIUS)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    PauseButton::Home,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Back to Home"),
                        TextFont {
                            font_size: FontSize::Px(20.0),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // Exit Button
            parent
                .spawn((
                    Node {
                        width: Val::Px(BUTTON_WIDTH),
                        height: Val::Px(BUTTON_HEIGHT),
                        padding: UiRect::all(Val::Px(CORNER_RADIUS)),
                        border: UiRect::all(Val::Px(BORDER_WIDTH)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(Val::Px(CORNER_RADIUS)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    PauseButton::Exit,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Exit"),
                        TextFont {
                            font_size: FontSize::Px(20.0),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

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

fn handle_pause_buttons(
    mut interaction_query: Query<
        (&PickingInteraction, &PauseButton, &mut BackgroundColor),
        Changed<PickingInteraction>,
    >,
    mut app_state: ResMut<NextState<SceneState>>,
    mut game_state: ResMut<NextState<PauseState>>,
    mut setting_state: ResMut<NextState<SettingState>>,
    mut exit: MessageWriter<AppExit>,
    mut save_events: MessageWriter<SaveWorldEvent>,
    mut load_events: MessageWriter<LoadWorldEvent>,
    mut refresh: MessageWriter<RefreshSlotPanel>,
) {
    for (interaction, button_type, mut bg_color) in &mut interaction_query {
        match *interaction {
            PickingInteraction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.35, 0.75, 0.35));
                match button_type {
                    PauseButton::Resume => {
                        game_state.set(PauseState::Playing);
                    }
                    PauseButton::Settings => {
                        setting_state.set(SettingState::Open);
                    }
                    PauseButton::Home => {
                        game_state.set(PauseState::Playing);
                        app_state.set(SceneState::Home);
                    }
                    PauseButton::Exit => {
                        game_state.set(PauseState::Playing);
                        exit.write(AppExit::Success);
                    }
                    PauseButton::Save => {
                        save_events.write(SaveWorldEvent { slot: 1 });
                        refresh.write(RefreshSlotPanel);
                    }
                    PauseButton::Load => {
                        game_state.set(PauseState::Playing);
                        // Scene teardown is handled by `DespawnOnExit(SceneState::Game)`.
                        load_events.write(LoadWorldEvent { slot: 1 });
                    }
                }
            }
            PickingInteraction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.25, 0.25, 0.25));
            }
            PickingInteraction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.15));
            }
        }
    }
}

fn cleanup_pause(mut time: ResMut<Time<Virtual>>) {
    time.unpause();
    // UI entities are removed by `DespawnOnExit(PauseState::Paused)`.
}
