use std::time::Duration;

use crate::{
    constant::{GRID_SIZE, SMALL_GRID_SIZE},
    content::{
        block::base::{Block, BlockOwner, BlockRegistry, Health},
        ship::base::{BoundingBox, Ship},
        team::Team,
    },
    resource::{GameVars, SceneEntity, SceneState},
    scene::background::BackgroundPlugin,
};

use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
    time::common_conditions::on_timer,
};

#[derive(Component)]
pub struct FpsText;

#[derive(Component)]
pub struct ShipButton(pub Entity);

#[derive(Component)]
pub struct ShipList;

#[derive(Component)]
pub struct BlockButton(pub Entity);

pub struct GameScenePlugin;

impl Plugin for GameScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameVars>()
            .add_systems(OnEnter(SceneState::Game), setup)
            .add_systems(
                Update,
                (
                    update_fps,
                    update_ship_buttons.run_if(on_timer(Duration::from_millis(200))),
                    handle_ship_button_click,
                )
                    .run_if(in_state(SceneState::Game)),
            )
            .add_plugins(BackgroundPlugin);
    }
}

pub fn setup(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Stretch,
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },
            SceneEntity(SceneState::Game),
        ))
        .with_children(|game_ui| {
            game_ui
                .spawn((Node {
                    width: Val::Percent(100.0),
                    min_height: Val::Px(40.0),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                },))
                .with_children(|top_bar| {
                    top_bar.spawn((
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

            game_ui
                .spawn((Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                },))
                .with_children(|central_part| {
                    central_part
                        .spawn((Node {
                            width: Val::Px(40.0),
                            min_height: Val::Percent(100.0),
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },))
                        .with_children(|left_bar| {
                            left_bar.spawn((
                                ShipList,
                                Node {
                                    flex_direction: FlexDirection::Column,
                                    flex_wrap: FlexWrap::Wrap,
                                    ..default()
                                },
                            ));
                        });

                    central_part.spawn((Node {
                        width: Val::Px(40.0),
                        min_height: Val::Percent(100.0),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },));
                });

            game_ui.spawn((Node {
                width: Val::Percent(100.0),
                min_height: Val::Px(40.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },));
        });
}

pub fn update_fps(
    mut text: Single<&mut Text, With<FpsText>>,
    diagnostics: Res<DiagnosticsStore>,
    camera: Single<&GlobalTransform, With<Camera2d>>,
) {
    let position = camera.translation().truncate();

    let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) else {
        return;
    };

    if let Some(value) = fps.smoothed() {
        text.0 = format!("FPS: {value:.2}, {position}",);
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
    block_list: Query<(Entity, &Block, &Health, &BlockOwner)>,
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
                        .filter(|(_, _, _, owner)| owner.id == *entity)
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
                        .filter(|(_, _, _, owner)| owner.id == *entity)
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

    if hp > 0.75 {
        Color::srgb(0.0, 1.0, 0.0)
    } else if hp > 0.5 {
        Color::srgb(1.0, 1.0, 0.0)
    } else {
        Color::srgb(1.0, 0.0, 0.0)
    }
}
