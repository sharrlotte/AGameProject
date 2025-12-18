use bevy::prelude::*;

use crate::AppState;

pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Pause), setup_pause)
            .add_systems(Update, handle_pause_buttons.run_if(in_state(AppState::Pause)))
            .add_systems(OnExit(AppState::Pause), cleanup_pause);
    }
}

#[derive(Component)]
struct PauseScreen;

#[derive(Component)]
enum PauseButton {
    Resume,
    Home,
    Exit,
}

fn setup_pause(mut commands: Commands, mut time: ResMut<Time<Virtual>>) {
    info!("Setting up Pause Menu");

    time.pause();

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
    mut next_state: ResMut<NextState<AppState>>,
    mut exit: MessageWriter<AppExit>,
) {
    for (interaction, button_type, mut bg_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.35, 0.75, 0.35));
                match button_type {
                    PauseButton::Resume => {
                        info!("Resume button clicked");
                        next_state.set(AppState::Game);
                    }
                    PauseButton::Home => {
                        info!("Home button clicked");
                        next_state.set(AppState::Home);
                    }
                    PauseButton::Exit => {
                        info!("Exit button clicked");
                        exit.write(AppExit::Success);
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

fn cleanup_pause(mut commands: Commands, query: Query<Entity, With<PauseScreen>>, mut time: ResMut<Time<Virtual>>) {
    info!("Cleaning up Pause Menu");
    time.unpause();
    for entity in &query {
        commands.entity(entity).despawn();
    }
}