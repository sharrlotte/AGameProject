use bevy::prelude::*;

use crate::{SceneState, resource::SettingState};

pub struct HomeScenePlugin;

impl Plugin for HomeScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(SceneState::Home), setup_home)
            .add_systems(Update, handle_home_buttons.run_if(in_state(SceneState::Home)))
            .add_systems(OnExit(SceneState::Home), cleanup_home);
    }
}

#[derive(Component)]
struct HomeScreen;

#[derive(Component)]
enum HomeButton {
    Play,
    Settings,
    Exit,
    ShipEditor,
}

fn setup_home(mut commands: Commands) {
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
            HomeScreen,
        ))
        .with_children(|parent| {
            // Play Button
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
                    HomeButton::Play,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Play"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // Ship Editor Button (New)
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
                    HomeButton::ShipEditor,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Ship Editor"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // Settings Button
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
                    HomeButton::Settings,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Settings"),
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
                    HomeButton::Exit,
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

fn handle_home_buttons(
    mut interaction_query: Query<
        (&Interaction, &HomeButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<SceneState>>,
    mut next_setting_state: ResMut<NextState<SettingState>>,
    // Use AppExit event directly to avoid EventWriter deprecation/method issues
    mut exit: MessageWriter<AppExit>,
) {
    for (interaction, button_type, mut bg_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.35, 0.75, 0.35));
                match button_type {
                    HomeButton::Play => {
                        next_state.set(SceneState::Game);
                    }
                    HomeButton::ShipEditor => {
                        next_state.set(SceneState::ShipEditor);
                    }
                    HomeButton::Settings => {
                        next_setting_state.set(SettingState::Open);
                    }
                    HomeButton::Exit => {
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

fn cleanup_home(mut commands: Commands, query: Query<Entity, With<HomeScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
