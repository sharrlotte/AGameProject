use std::time::Duration;

use crate::{
    constant::{GRID_SIZE, SMALL_GRID_SIZE},
    content::{
        block::{Block, BlockOwner, BlockRegistry, Health},
        selectable::EntitySelectEvent,
        ship::components::{BoundingBox, Ship},
        team::Team,
    },
    resource::{CameraTarget, ControlledShip, SceneEntity, SceneState},
    scene::background::BackgroundPlugin,
    simulation::SimulationSet,
};

use bevy::{
    picking::hover::PickingInteraction,
    prelude::*,
    time::common_conditions::on_timer,
};

#[cfg(debug_assertions)]
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};

#[cfg(debug_assertions)]
#[derive(Component)]
pub struct FpsText;

#[derive(Component)]
pub struct ShipButton(pub Entity);

#[derive(Component)]
pub struct ShipList;


#[derive(Component)]
pub struct SelectablePanel;

pub struct GameScenePlugin;

impl Plugin for GameScenePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(SceneState::Game), setup)
            .add_systems(
                Update,
                (
                    #[cfg(debug_assertions)]
                    update_fps,
                    update_ship_list.run_if(on_timer(Duration::from_millis(200))),
                    handle_ship_button_click,
                    update_select_panel,
                )
                    .run_if(in_state(SceneState::Game))
                    .in_set(SimulationSet::Input),
            )
            .add_plugins(BackgroundPlugin);
    }
}

fn setup(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Stretch,
                flex_direction: FlexDirection::Column,
                overflow: Overflow::hidden(),
                ..Default::default()
            },
            SceneEntity(SceneState::Game),
        ))
        .with_children(|game_ui| {
            #[cfg(debug_assertions)]
            game_ui
                .spawn((Node {
                    min_height: Val::Px(40.0),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                },))
                .with_children(|top_bar| {
                    top_bar.spawn((
                        Text::new("FPS: N/A"),
                        TextFont {
                            font_size: FontSize::Px(10.0),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Node { ..default() },
                        FpsText,
                    ));
                });

            game_ui
                .spawn((Node {
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    overflow: Overflow::hidden(),
                    ..default()
                },))
                .with_children(|central_part| {
                    central_part
                        .spawn((Node {
                            height: Val::Percent(100.0),
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            flex_direction: FlexDirection::Column,
                            overflow: Overflow::hidden(),
                            ..default()
                        },))
                        .with_children(|left_bar| {
                            left_bar.spawn((
                                ShipList,
                                Node {
                                    height: Val::Percent(100.0),
                                    flex_direction: FlexDirection::Column,
                                    flex_wrap: FlexWrap::Wrap,
                                    ..default()
                                },
                            ));
                        });

                    central_part.spawn((Node {
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },));
                });

            game_ui
                .spawn((Node {
                    min_height: Val::Px(40.0),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                },))
                .with_children(|bottom_bar| {
                    bottom_bar.spawn((
                        SelectablePanel,
                        Node {
                            flex_wrap: FlexWrap::Wrap,
                            row_gap: Val::Px(5.0),
                            column_gap: Val::Px(5.0),
                            ..default()
                        },
                    ));
                });
        });
}

fn update_select_panel(
    selectable_panel: Single<Entity, With<SelectablePanel>>,
    mut commands: Commands,
    mut selectable_reader: MessageReader<EntitySelectEvent>,
) {
    for event in selectable_reader.read() {
        commands
            .entity(selectable_panel.entity())
            .despawn_children();

        commands
            .entity(selectable_panel.entity())
            .with_children(|parent| {
                parent.spawn((
                    Node::default(),
                    Text::new(format!("Selected: {}", event.selected.len())),
                ));
            });
    }
}

#[cfg(debug_assertions)]
fn update_fps(
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

fn handle_ship_button_click(
    mut interaction: Query<(&PickingInteraction, &ShipButton), Changed<PickingInteraction>>,
    mut camera_target: ResMut<CameraTarget>,
    mut controlled: ResMut<ControlledShip>,
) {
    for (interaction, ship_button) in interaction.iter_mut() {
        if *interaction == PickingInteraction::Pressed {
            if let Some(following) = camera_target.following
                && following == ship_button.0
            {
                if let Some(controlling) = controlled.controlling
                    && controlling == ship_button.0
                {
                    camera_target.following = None;
                    controlled.controlling = None;
                } else {
                    controlled.controlling = Some(ship_button.0);
                }
            } else {
                camera_target.following = Some(ship_button.0);
                controlled.controlling = None;
            }
        }
    }
}

fn update_ship_list(
    mut commands: Commands,
    mut ship_list: Query<Entity, With<ShipList>>,
    ships: Query<(Entity, &Ship, &BoundingBox, &Team)>,
    block_list: Query<(Entity, &Block, &Health, &BlockOwner)>,
    state: Res<ControlledShip>,
    camera_target: Res<CameraTarget>,
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
            } else if let Some(following) = camera_target.following
                && following == *entity
            {
                Color::srgba(0.0, 1.0, 1.0, 1.0)
            } else {
                Color::WHITE
            };

            container
                .spawn((
                    Node {
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(4.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Percent(5.0)),
                        ..Default::default()
                    },
                    BorderColor::all(color),

                    ShipButton(*entity),
                ))
                .with_children(|wrapper| {
                    wrapper
                        .spawn((Node {
                            position_type: PositionType::Relative,
                            min_width: Val::Px((bbox.size.x / GRID_SIZE + 2.0) * SMALL_GRID_SIZE),
                            height: Val::Px((bbox.size.y / GRID_SIZE + 2.0) * SMALL_GRID_SIZE),
                            ..default()
                        },))
                        .with_children(|block_container| {
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

                            for (_, block, health, _) in block_list
                                .iter()
                                .filter(|(_, _, _, owner)| owner.id == *entity)
                            {
                                let Some(def) = registry.get(&block.block_type) else {
                                    continue;
                                };

                                let hp_percent =
                                    (health.health / health.max_health).clamp(0.0, 1.0);

                                let color = if hp_percent > 0.75 {
                                    Color::srgb(0.0, 1.0, 0.0)
                                } else if hp_percent > 0.5 {
                                    Color::srgb(1.0, 1.0, 0.0)
                                } else {
                                    Color::srgb(1.0, 0.0, 0.0)
                                };

                                block_container.spawn((
                                    BackgroundColor(color),
                                    Node {
                                        position_type: PositionType::Absolute,
                                        top: Val::Px(
                                            ((block.position.y - offset_y) as f32)
                                                * SMALL_GRID_SIZE
                                                + SMALL_GRID_SIZE,
                                        ),
                                        left: Val::Px(
                                            ((block.position.x - offset_x) as f32)
                                                * SMALL_GRID_SIZE
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
        });
    }
}
