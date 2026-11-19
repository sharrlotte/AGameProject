use super::*;

use crate::scene::widget::{ButtonVisuals, spawn_button, spawn_widget};

pub fn setup_editor_ui(
    mut commands: Commands,
    registry: Res<BlockRegistry>,
    mut camera: Query<&mut Transform, With<Camera2d>>,
) {
    if let Ok(mut transform) = camera.single_mut() {
        transform.translation = Vec3::new(0.0, 0.0, 0.0);
    }
    let mut block_buttons = Vec::new();
    commands.spawn((
        ShipBase,
        Visibility::Inherited,
        Transform::default(),
        DespawnOnExit(SceneState::ShipEditor),
    ));

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            EditorUiRoot,
            DespawnOnExit(SceneState::ShipEditor),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        width: Val::Px(PANEL_WIDTH),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(CORNER_RADIUS)),
                        border: UiRect::right(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
                ))
                .with_children(|left_panel| {
                    spawn_button(
                        left_panel,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(ROW_HEIGHT),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            margin: UiRect::bottom(Val::Px(CORNER_RADIUS)),
                            border_radius: BorderRadius::all(Val::Px(6.0)),
                            ..default()
                        },
                        "Back to Home",
                        14.0,
                        Color::WHITE,
                        NavBackButton,
                        on_nav_back,
                        ButtonVisuals::dark(),
                    );

                    left_panel
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(ROW_HEIGHT),
                            margin: UiRect::bottom(Val::Px(CORNER_RADIUS)),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        })
                        .with_children(|header| {
                            spawn_button(
                                header, tab_node(), "Blocks", 16.0, Color::WHITE,
                                TabButton(EditorTab::Blocks), on_tab, ButtonVisuals::dark(),
                            );
                            spawn_button(
                                header, tab_node(), "Ships", 16.0, Color::WHITE,
                                TabButton(EditorTab::Ships), on_tab, ButtonVisuals::dark(),
                            );
                            spawn_button(
                                header, tab_node(), "AI", 16.0, Color::WHITE,
                                TabButton(EditorTab::Ai), on_tab, ButtonVisuals::dark(),
                            );
                        });

                    left_panel
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Column,
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                row_gap: Val::Px(6.0),
                                display: Display::None,
                                ..default()
                            },
                            AiPanel,
                        ))
                        .with_children(|panel| {
                            panel.spawn((
                                Text::new("AI Doctrine"),
                                TextFont {
                                    font_size: FontSize::Px(16.0),
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                            for doctrine in Doctrine::ALL {
                                spawn_button(
                                    panel,
                                    Node {
                                        width: Val::Percent(100.0),
                                        height: Val::Px(ROW_HEIGHT),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        border_radius: BorderRadius::all(Val::Px(4.0)),
                                        ..default()
                                    },
                                    doctrine.label(),
                                    13.0,
                                    Color::WHITE,
                                    DoctrineButton(doctrine),
                                    on_doctrine,
                                    ButtonVisuals::green(),
                                );
                            }
                        });

                    left_panel
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Column,
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                row_gap: Val::Px(8.0),
                                display: Display::Flex,
                                overflow: Overflow::scroll_y(),
                                ..default()
                            },
                            BlockListPanel,
                        ))
                        .with_children(|list_content| {
                            for (block_type, def) in registry.iter() {
                                let button = spawn_widget(
                                    list_content,
                                    Node {
                                        width: Val::Percent(100.0),
                                        height: Val::Px(60.0),
                                        justify_content: JustifyContent::Start,
                                        align_items: AlignItems::Center,
                                        border: UiRect::all(Val::Px(1.0)),
                                        padding: UiRect::all(Val::Px(8.0)),
                                        border_radius: BorderRadius::all(Val::Px(8.0)),
                                        ..default()
                                    },
                                    BlockSelectButton(block_type.clone()),
                                    on_block_select,
                                    ButtonVisuals::dark(),
                                );
                                block_buttons.push((button, def));
                            }
                        });

                    left_panel.spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            width: Val::Percent(100.0),
                            row_gap: Val::Px(8.0),
                            display: Display::None,
                            ..default()
                        },
                        ShipListPanel,
                    ));
                });

            parent.spawn(Node {
                flex_grow: 1.0,
                height: Val::Percent(100.0),
                ..default()
            });

            parent
                .spawn((
                    Node {
                        width: Val::Px(BUTTON_WIDTH),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(CORNER_RADIUS)),
                        border: UiRect::left(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
                ))
                .with_children(|right_panel| {
                    right_panel.spawn((
                        Text::new("Properties"),
                        TextFont {
                            font_size: FontSize::Px(20.0),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Node {
                            margin: UiRect::bottom(Val::Px(CORNER_RADIUS)),
                            ..default()
                        },
                    ));

                    right_panel.spawn((
                        Text::new("Select a block..."),
                        TextFont {
                            font_size: FontSize::Px(14.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                        BlockPropertiesText,
                    ));

                    right_panel.spawn((
                        Text::new("\n\nLeft Click: Place\nRight Click: Delete"),
                        TextFont {
                            font_size: FontSize::Px(12.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.6, 0.6, 0.6)),
                        Node {
                            margin: UiRect::top(Val::Px(FONT_TITLE)),
                            ..default()
                        },
                    ));

                    right_panel.spawn(Node {
                        flex_grow: 1.0,
                        ..default()
                    });

                    spawn_button(
                        right_panel,
                        action_node(),
                        "Deploy",
                        16.0,
                        Color::WHITE,
                        DeployButton,
                        on_deploy,
                        ButtonVisuals::blue(),
                    );

                    spawn_button(
                        right_panel,
                        action_node(),
                        "Start Game",
                        16.0,
                        Color::WHITE,
                        NavPlayButton,
                        on_nav_play,
                        ButtonVisuals::green(),
                    );
                });
        });

    for (button, def) in block_buttons {
        commands.entity(button).with_children(|btn| {
            btn.spawn((
                Node {
                    width: Val::Px(ICON_SIZE),
                    height: Val::Px(ICON_SIZE),
                    flex_shrink: 0.0,
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|icon_parent| {
                for feature in &def.features {
                    match feature {
                        Feature::Texture(visual) => {
                            icon_parent.spawn((
                                ImageNode {
                                    image: visual.image.clone(),
                                    ..default()
                                },
                                Node {
                                    position_type: PositionType::Absolute,
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(100.0),
                                    ..default()
                                },
                                Pickable::IGNORE,
                            ));
                        }
                        Feature::Turret(visual) => {
                            icon_parent.spawn((
                                ImageNode {
                                    image: visual.image.clone(),
                                    ..default()
                                },
                                Node {
                                    position_type: PositionType::Absolute,
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(100.0),
                                    ..default()
                                },
                                Pickable::IGNORE,
                            ));
                        }
                        _ => {}
                    }
                }
            });

            btn.spawn((Node {
                width: Val::Px(FONT_SMALL),
                ..default()
            }, Pickable::IGNORE));

            btn.spawn((
                Text::new(&def.name),
                TextFont { font_size: FontSize::Px(14.0), ..default() },
                TextColor(Color::WHITE),
                Pickable::IGNORE,
            ));
        });
    }
}

fn tab_node() -> Node {
    Node {
        width: Val::Percent(32.0),
        height: Val::Percent(100.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    }
}

fn action_node() -> Node {
    Node {
        width: Val::Percent(100.0),
        height: Val::Px(ROW_HEIGHT),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        margin: UiRect::top(Val::Px(CORNER_RADIUS)),
        border_radius: BorderRadius::all(Val::Px(6.0)),
        ..default()
    }
}
