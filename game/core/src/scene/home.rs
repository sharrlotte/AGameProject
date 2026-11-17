use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;

use crate::resource::{SceneState, SettingState};
use crate::scene::slot_panel::SlotPanel;
use crate::simulation::SimulationSet;

pub struct HomeScenePlugin;

impl Plugin for HomeScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(SceneState::Home), setup_home)
            .add_systems(
                Update,
                handle_home_buttons
                    .run_if(in_state(SceneState::Home))
                    .in_set(SimulationSet::Input),
            );
    }
}

#[derive(Component)]
struct HomeScreen;

#[derive(Component)]
enum HomeButton {
    Play,
    QuickBattle,
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
            DespawnOnExit(SceneState::Home),
        ))
        .with_children(|parent| {
            // Play Button
            parent
                .spawn((
                    Node {
                        width: Val::Px(250.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(5.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(10.0)),
                        border_radius: BorderRadius::all(Val::Px(10.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    HomeButton::Play,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Play"),
                        TextFont {
                            font_size: FontSize::Px(20.0),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // Quick Battle Button
            parent
                .spawn((
                    Node {
                        width: Val::Px(250.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(5.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(10.0)),
                        border_radius: BorderRadius::all(Val::Px(10.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.35, 0.55)),
                    HomeButton::QuickBattle,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Quick Battle"),
                        TextFont {
                            font_size: FontSize::Px(20.0),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

            // Ship Editor Button (New)
            parent
                .spawn((
                    Node {
                        width: Val::Px(250.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(5.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(10.0)),
                        border_radius: BorderRadius::all(Val::Px(10.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    HomeButton::ShipEditor,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Ship Editor"),
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
                        width: Val::Px(250.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(5.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(10.0)),
                        border_radius: BorderRadius::all(Val::Px(10.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    HomeButton::Settings,
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

            // Exit Button
            parent
                .spawn((
                    Node {
                        width: Val::Px(250.0),
                        height: Val::Px(65.0),
                        padding: UiRect::all(Val::Px(10.0)),
                        border: UiRect::all(Val::Px(5.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(Val::Px(10.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    HomeButton::Exit,
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
                    SlotPanel { allow_save: false },
                ))
                .with_children(|panel| {
                    crate::scene::slot_panel::build_slot_panel(panel, false);
                });
        });
}

fn handle_home_buttons(
    mut interaction_query: Query<
        (&PickingInteraction, &HomeButton, &mut BackgroundColor),
        Changed<PickingInteraction>,
    >,
    mut next_state: ResMut<NextState<SceneState>>,
    mut next_setting_state: ResMut<NextState<SettingState>>,
    // Use AppExit event directly to avoid EventWriter deprecation/method issues
    mut exit: MessageWriter<AppExit>,
) {
    for (interaction, button_type, mut bg_color) in &mut interaction_query {
        match *interaction {
            PickingInteraction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.35, 0.75, 0.35));
                match button_type {
                    HomeButton::Play => {
                        next_state.set(SceneState::Game);
                    }
                    HomeButton::QuickBattle => {
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
            PickingInteraction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.25, 0.25, 0.25));
            }
            PickingInteraction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.15));
            }
        }
    }
}
