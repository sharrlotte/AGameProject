use bevy::prelude::*;

use crate::{
    AppState, GameState,
    save::{
        base::{LoadWorldEvent, SaveWorldEvent},
        io::save_dir,
    },
};

pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Paused), setup_pause)
            .add_systems(
                Update,
                handle_pause_buttons.run_if(in_state(GameState::Paused)),
            )
            .add_systems(OnExit(GameState::Paused), cleanup_pause)
            .add_systems(
                Update,
                handle_pause_input
                    .run_if(in_state(AppState::Game).or(in_state(AppState::ShipEditor))),
            );
    }
}

#[derive(Component)]
enum PauseButton {
    Resume,
    Home,
    Save,
    Load,
    Exit,
}

#[derive(Component)]
struct PauseScreen;

fn handle_pause_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut game_state: ResMut<NextState<GameState>>,
    state: Res<State<GameState>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        let next_state: GameState = match state.get() {
            GameState::Paused => GameState::Playing,
            GameState::Playing => GameState::Paused,
        };

        game_state.set(next_state);
    }
}

fn setup_pause(mut commands: Commands, mut time: ResMut<Time<Virtual>>) {
    info!("Setting up Pause Menu");

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
                row_gap: Val::Px(20.0),
                ..default()
            },
            PauseScreen,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Paused"),
                TextFont {
                    font_size: 40.0,
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
                    Button,
                    Node {
                        width: Val::Px(250.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(5.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    BorderRadius::all(Val::Px(10.0)),
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    PauseButton::Resume,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Resume"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // Save Button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(250.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(5.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    BorderRadius::all(Val::Px(10.0)),
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    PauseButton::Save,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Save"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(250.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(5.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    BorderRadius::all(Val::Px(10.0)),
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    PauseButton::Load,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Load"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // Home Button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(250.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(5.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    BorderRadius::all(Val::Px(10.0)),
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    PauseButton::Home,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Back to Home"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // Exit Button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(250.0),
                        height: Val::Px(65.0),
                        padding: UiRect::all(Val::Px(10.0)),
                        border: UiRect::all(Val::Px(5.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderRadius::all(Val::Px(10.0)),
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    PauseButton::Exit,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Exit"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

fn handle_pause_buttons(
    mut interaction_query: Query<
        (&Interaction, &PauseButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_state: ResMut<NextState<AppState>>,
    mut game_state: ResMut<NextState<GameState>>,
    mut exit: MessageWriter<AppExit>,
    mut save_events: MessageWriter<SaveWorldEvent>,
    mut load_events: MessageWriter<LoadWorldEvent>,
) {
    for (interaction, button_type, mut bg_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.35, 0.75, 0.35));
                match button_type {
                    PauseButton::Resume => {
                        game_state.set(GameState::Playing);
                    }
                    PauseButton::Home => {
                        app_state.set(AppState::Home);
                    }
                    PauseButton::Exit => {
                        exit.write(AppExit::Success);
                    }
                    PauseButton::Save => {
                        save_events.write(SaveWorldEvent {
                            path: save_dir().join("saves").join("test.json"),
                        });
                    }
                    PauseButton::Load => {
                        load_events.write(LoadWorldEvent {
                            path: save_dir().join("saves").join("test.json"),
                        });
                    }
                }
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.25, 0.25, 0.25));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.15));
            }
        }
    }
}

fn cleanup_pause(
    mut commands: Commands,
    query: Query<Entity, With<PauseScreen>>,
    mut time: ResMut<Time<Virtual>>,
) {
    info!("Cleaning up Pause Menu");
    time.unpause();
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
