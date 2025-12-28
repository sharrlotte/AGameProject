use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

use crate::GameState;
use crate::constant::{GRID_SIZE, SMALL_GRID_SIZE};
use crate::content::block::component::{Block, Health, ShipIdComponent};
use crate::content::block::resource::BlockRegistry;
use crate::content::ship::component::{BoundingBox, Ship};
use crate::content::team::Team;
use crate::resource::TurretControl;
use crate::scene::component::SceneItem;
use crate::scene::game::component::{
    BlockButton, FpsText, ShipButton, ShipList, TurretControlModeButton, TurretControlModeText,
};
use crate::scene::game::resource::GameVars;

pub fn update_fps(
    diagnostics: Res<DiagnosticsStore>,
    mut text: Query<&mut Text, With<FpsText>>,
    camera: Query<&GlobalTransform, With<Camera2d>>,
) {
    let position = camera
        .single()
        .expect("No camera attached")
        .translation()
        .truncate();

    if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(value) = fps.smoothed() {
            for mut t in text.iter_mut() {
                t.0 = format!("FPS: {value:.2}, {position}",);
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
            if let Some(following) = state.following
                && following == ship_button.0
            {
                if let Some(controlling) = state.controlling
                    && controlling == ship_button.0
                {
                    state.following = None;
                    state.controlling = None;
                } else {
                    state.controlling = Some(ship_button.0);
                }
            } else {
                state.following = Some(ship_button.0);
                state.controlling = None;
            }
        }
    }
}

pub fn update_ship_buttons(
    mut commands: Commands,
    ships: Query<(Entity, &Ship, &BoundingBox, &Team)>,
    mut ship_list: Query<Entity, With<ShipList>>,
    block_list: Query<(Entity, &Block, &Health, &ShipIdComponent)>,
    state: Res<GameVars>,
    registry: Res<BlockRegistry>,
) {
    let Ok(ship_list) = ship_list.single_mut() else {
        return;
    };

    commands.entity(ship_list).despawn_children();

    let mut player_ships = ships
        .iter()
        .filter(|(_, _, _, team)| team.is_player())
        .collect::<Vec<_>>();

    player_ships.sort_by(|a, b| a.1.id.cmp(&b.1.id));

    for (entity, _, bbox, _) in player_ships.iter() {
        commands.entity(ship_list).with_children(|container| {
            let color: Color = if let Some(controlling) = state.controlling
                && controlling == *entity
            {
                Color::srgba(0.0, 1.0, 0.0, 1.0)
            } else if let Some(following) = state.following
                && following == *entity
            {
                Color::srgba(0.0, 1.0, 1.0, 1.0)
            } else {
                Color::srgba(1.0, 1.0, 1.0, 0.0)
            };

            container
                .spawn((
                    Node {
                        position_type: PositionType::Relative,
                        width: Val::Percent(100.0),
                        min_width: Val::Px((bbox.size.x / GRID_SIZE + 2.0) * SMALL_GRID_SIZE),
                        height: Val::Px((bbox.size.y / GRID_SIZE + 2.0) * SMALL_GRID_SIZE),
                        margin: UiRect::all(Val::Px(4.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BorderColor::from(Color::WHITE),
                    BorderRadius::all(Val::Percent(5.0)),
                    Button,
                    ShipButton(*entity),
                    BackgroundColor(color),
                ))
                .with_children(|ship_ui| {
                    let mut offset_x = 0;
                    let mut offset_y = 0;

                    for (_, block, _, _) in block_list
                        .iter()
                        .filter(|(_, _, _, ship_id)| ship_id.id == *entity)
                    {
                        if block.position.x < offset_x {
                            offset_x = block.position.x;
                        }

                        if block.position.y < offset_y {
                            offset_y = block.position.y;
                        }
                    }

                    for (block_entity, block, health, _) in block_list
                        .iter()
                        .filter(|(_, _, _, ship_id)| ship_id.id == *entity)
                    {
                        let Some(def) = registry.get(&block.block_type) else {
                            continue;
                        };

                        let hp_percent = health.health / health.max_health;
                        let color = hp_to_color(hp_percent);

                        ship_ui.spawn((
                            BlockButton(block_entity),
                            BackgroundColor(color),
                            Node {
                                position_type: PositionType::Absolute,
                                top: Val::Px(
                                    ((block.position.y - offset_y) as f32) * SMALL_GRID_SIZE
                                        + SMALL_GRID_SIZE,
                                ),
                                left: Val::Px(
                                    ((block.position.x - offset_x) as f32) * SMALL_GRID_SIZE
                                        + SMALL_GRID_SIZE,
                                ),
                                width: Val::Px(def.get_block_size().x * SMALL_GRID_SIZE),
                                height: Val::Px(def.get_block_size().y * SMALL_GRID_SIZE),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BorderColor::from(Color::WHITE),
                        ));
                    }
                });
        });
    }
}

fn hp_to_color(hp_percent: f32) -> Color {
    let hp = hp_percent.clamp(0.0, 1.0);

    let green = Color::srgb(0.0, 1.0, 0.0);
    let yellow = Color::srgb(1.0, 1.0, 0.0);
    let red = Color::srgb(1.0, 0.0, 0.0);

    if hp > 0.75 {
        green
    } else if hp > 0.5 {
        yellow
    } else {
        red
    }
}

pub fn setup_game(mut commands: Commands, game_state: Res<GameState>) {
    commands.spawn((
        ShipList,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(40.0),
            left: Val::Px(10.0),
            height: Val::Vh(90.0),
            flex_direction: FlexDirection::Column,
            flex_wrap: FlexWrap::Wrap,
            ..default()
        },
        SceneItem,
    ));

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                ..default()
            },
            SceneItem,
        ))
        .with_children(|container| {
            let text = match game_state.turret_control {
                TurretControl::Manual => "Turret Control: Manual",
                TurretControl::Auto => "Turret Control: Automatic",
                TurretControl::Hold => "Turret Control: Hold",
            };

            container
                .spawn((
                    Node {
                        padding: UiRect::all(Val::Px(5.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    Button,
                    TurretControlModeButton,
                    BorderRadius::all(Val::Px(5.0)),
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new(text),
                        TextFont {
                            font_size: 10.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        TurretControlModeText,
                    ));
                });

            container.spawn((
                Text::new("FPS: N/A"),
                TextFont {
                    font_size: 10.0,
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
