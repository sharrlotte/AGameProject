use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

use crate::GameState;
use crate::content::bullet::component::Bullet;
use crate::content::ship::component::Ship;
use crate::content::team::Team;
use crate::resource::TurretControl;
use crate::scene::game::background::{BackgroundConfig, BackgroundImage};
use crate::scene::game::component::{
    FpsText, ShipButton, ShipList, TurretControlModeButton, TurretControlModeText,
};
use crate::scene::game::resource::GameVars;

pub fn update_fps(diagnostics: Res<DiagnosticsStore>, mut text: Query<&mut Text, With<FpsText>>) {
    if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(value) = fps.smoothed() {
            for mut t in text.iter_mut() {
                t.0 = format!("FPS: {value:.2}",);
            }
        }
    }
}

pub fn handle_ship_button_click(
    mut interaction: Query<(&Interaction, &ShipButton), (Changed<Interaction>, With<Button>)>,
    mut state: ResMut<GameVars>,
) {
    for (interaction, ship_button) in interaction.iter_mut() {
        if *interaction == Interaction::Pressed {
            if let Some(controlling) = state.controlling
                && controlling == ship_button.0
            {
                state.controlling = None;
            } else {
                state.controlling = Some(ship_button.0);
            }

            if let Some(following) = state.following
                && following == ship_button.0
            {
                state.following = None;
            } else {
                state.following = Some(ship_button.0);
            }
        }
    }
}

pub fn update_ship_buttons(
    mut commands: Commands,
    ships: Query<(Entity, &Ship, &Team)>,
    mut ship_list: Query<Entity, With<ShipList>>,
) {
    let Ok(ship_list_entity) = ship_list.single_mut() else {
        return;
    };

    commands.entity(ship_list_entity).despawn_children();

    for (entity, ship, team) in ships.iter() {
        if !team.is_player() {
            continue;
        }

        commands
            .entity(ship_list_entity)
            .with_children(|container| {
                container.spawn((
                    Text::new(format!("Ship: {}", ship.id)),
                    TextFont {
                        font_size: 8.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Node {
                        padding: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    Button,
                    ShipButton(entity),
                ));
            });
    }
}

pub fn setup_game(
    mut commands: Commands,
    game_state: Res<GameState>,
    fps_query: Query<(), With<FpsText>>,
) {
    if fps_query.is_empty() {
        commands.spawn((
            Text::new("FPS: N/A"),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                ..default()
            },
            FpsText,
        ));
    }

    commands.spawn((
        ShipList,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(60.0),
            left: Val::Px(10.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
    ));

    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        })
        .with_children(|container| {
            let text = match game_state.turret_control {
                TurretControl::Manual => "Turret Control: Manual",
                TurretControl::Auto => "Turret Control: Automatic",
                TurretControl::Hold => "Turret Control: Hold",
            };

            container
                .spawn((
                    Node {
                        padding: UiRect::all(Val::Px(10.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    Button,
                    TurretControlModeButton,
                    BorderRadius::all(Val::Px(10.0)),
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new(text),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        TurretControlModeText,
                    ));
                });

            container.spawn((
                Text::new("FPS: N/A"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node { ..default() },
                FpsText,
            ));
        });
}

pub fn update_turret_control_mode_button(
    mut game_state: ResMut<GameState>,
    mut button_query: Query<&Interaction, (Changed<Interaction>, With<TurretControlModeButton>)>,
    mut text_query: Query<&mut Text, With<TurretControlModeText>>,
) {
    let Ok(interaction) = button_query.single_mut() else {
        return;
    };

    let Ok(mut text) = text_query.single_mut() else {
        return;
    };

    if *interaction == Interaction::Pressed {
        game_state.turret_control = game_state.turret_control.next();

        let mod_string = match game_state.turret_control {
            TurretControl::Manual => "Turret Control: Manual",
            TurretControl::Auto => "Turret Control: Automatic",
            TurretControl::Hold => "Turret Control: Hold",
        };
        text.0 = mod_string.to_string();
    }
}

pub fn cleanup_game(mut commands: Commands, query: Query<Entity, With<FpsText>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn cleanup_game_entities(
    mut commands: Commands,
    ships: Query<Entity, With<Ship>>,
    bullets: Query<Entity, With<Bullet>>,
    backgrounds: Query<Entity, With<BackgroundImage>>,
) {
    for entity in &ships {
        commands.entity(entity).despawn();
    }

    for entity in &bullets {
        commands.entity(entity).despawn();
    }

    for entity in &backgrounds {
        commands.entity(entity).despawn();
    }

    commands.remove_resource::<BackgroundConfig>();
}
