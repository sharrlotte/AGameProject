use super::*;

pub fn setup_editor_ui(
    mut commands: Commands,
    registry: Res<BlockRegistry>,
    mut camera: Query<&mut Transform, With<Camera2d>>,
) {
    if let Ok(mut transform) = camera.single_mut() {
        transform.translation = Vec3::new(0.0, 0.0, 0.0);
    }
    // Root Node (Full Screen)
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
            // Left Panel (Block Palette & Ships)
            parent
                .spawn((
                    Node {
                        width: Val::Px(PANEL_WIDTH), // Fixed width for better layout
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(CORNER_RADIUS)),
                        border: UiRect::right(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
                ))
                .with_children(|left_panel| {
                    // --- Back Button ---
                    left_panel
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(ROW_HEIGHT),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::bottom(Val::Px(CORNER_RADIUS)),
                                border_radius: BorderRadius::all(Val::Px(6.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.4, 0.4, 0.4)),
                            BackButton,
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("Back to Home"),
                                TextFont {
                                    font_size: FontSize::Px(14.0),
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });

                    // --- Tab Header ---
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
                            // Blocks Tab Button
                            header
                                .spawn((
                                    Node {
                                        width: Val::Percent(32.0),
                                        height: Val::Percent(100.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                                    TabButton(EditorTab::Blocks),
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new("Blocks"),
                                        TextFont {
                                            font_size: FontSize::Px(16.0),
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                });

                            // Ships Tab Button
                            header
                                .spawn((
                                    Node {
                                        width: Val::Percent(32.0),
                                        height: Val::Percent(100.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                                    TabButton(EditorTab::Ships),
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new("Ships"),
                                        TextFont {
                                            font_size: FontSize::Px(16.0),
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                });

                            // AI Tab Button
                            header
                                .spawn((
                                    Node {
                                        width: Val::Percent(32.0),
                                        height: Val::Percent(100.0),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                                    TabButton(EditorTab::Ai),
                                ))
                                .with_children(|btn| {
                                    btn.spawn((
                                        Text::new("AI"),
                                        TextFont {
                                            font_size: FontSize::Px(16.0),
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                });
                        });

                    // --- AI Panel ---
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
                                Text::new("AI Profile"),
                                TextFont {
                                    font_size: FontSize::Px(16.0),
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                            for preset in PRESETS {
                                panel
                                    .spawn((
                                        Node {
                                            width: Val::Percent(100.0),
                                            height: Val::Px(ROW_HEIGHT),
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            border_radius: BorderRadius::all(Val::Px(4.0)),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgb(0.2, 0.3, 0.2)),
                                        AiPresetButton(preset),
                                    ))
                                    .with_children(|btn| {
                                        btn.spawn((
                                            Text::new(*preset),
                                            TextFont {
                                                font_size: FontSize::Px(13.0),
                                                ..default()
                                            },
                                            TextColor(Color::WHITE),
                                        ));
                                    });
                            }
                            panel.spawn((
                                AiPreviewText,
                                Text::new(""),
                                TextFont {
                                    font_size: FontSize::Px(11.0),
                                    ..default()
                                },
                                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                            ));
                        });

                    // --- Block List Panel ---
                    left_panel
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Column,
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                row_gap: Val::Px(8.0),
                                display: Display::Flex, // Initially Visible
                                overflow: Overflow::scroll_y(),
                                ..default()
                            },
                            BlockListPanel,
                        ))
                        .with_children(|list_content| {
                            // Block List
                            for (block_type, def) in registry.iter() {
                                list_content
                                    .spawn((
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
                                        BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                                        BlockSelection(block_type.clone()),
                                    ))
                                    .with_children(|btn| {
                                        // Block Icon/Sprite
                                        // Create a container for the stacked layers
                                        btn.spawn(Node {
                                            width: Val::Px(ICON_SIZE),
                                            height: Val::Px(ICON_SIZE),
                                            flex_shrink: 0.0,
                                            ..default()
                                        })
                                        .with_children(
                                            |icon_parent| {
                                                for feature in &def.features {
                                                    match feature {
                                                        Feature::Texture(visual) => {
                                                            icon_parent.spawn((
                                                                ImageNode {
                                                                    image: visual.image.clone(),
                                                                    ..default()
                                                                },
                                                                Node {
                                                                    position_type:
                                                                        PositionType::Absolute,
                                                                    width: Val::Percent(100.0),
                                                                    height: Val::Percent(100.0),

                                                                    ..default()
                                                                },
                                                            ));
                                                        }
                                                        Feature::Turret(visual) => {
                                                            icon_parent.spawn((
                                                                ImageNode {
                                                                    image: visual.image.clone(),
                                                                    ..default()
                                                                },
                                                                Node {
                                                                    position_type:
                                                                        PositionType::Absolute,
                                                                    width: Val::Percent(100.0),
                                                                    height: Val::Percent(100.0),

                                                                    ..default()
                                                                },
                                                            ));
                                                        }
                                                        _ => {}
                                                    }
                                                }
                                            },
                                        );

                                        // Add a small spacer
                                        btn.spawn(Node {
                                            width: Val::Px(FONT_SMALL),
                                            ..default()
                                        });

                                        // Block Name
                                        btn.spawn((
                                            Text::new(&def.name),
                                            TextFont {
                                                font_size: FontSize::Px(14.0),
                                                ..default()
                                            },
                                            TextColor(Color::WHITE),
                                        ));
                                    });
                            }
                        });

                    // --- Ship List Panel ---
                    left_panel.spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            width: Val::Percent(100.0),
                            row_gap: Val::Px(8.0),
                            display: Display::None, // Initially Hidden
                            ..default()
                        },
                        ShipListPanel,
                    ));
                });

            // Center Panel (Transparent for Grid Interaction)
            parent.spawn(Node {
                flex_grow: 1.0,
                height: Val::Percent(100.0),
                ..default()
            });

            // Right Panel (Properties)
            parent
                .spawn((
                    Node {
                        width: Val::Px(BUTTON_WIDTH), // Fixed width
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

                    // Instructions
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

                    // Spacer
                    right_panel.spawn(Node {
                        flex_grow: 1.0,
                        ..default()
                    });

                    // Deploy Button
                    right_panel
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(ROW_HEIGHT),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::top(Val::Px(CORNER_RADIUS)),
                                border_radius: BorderRadius::all(Val::Px(6.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.2, 0.6, 1.0)),
                            DeployButton,
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("Deploy"),
                                TextFont {
                                    font_size: FontSize::Px(16.0),
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });

                    // Play Button
                    right_panel
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(ROW_HEIGHT),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::top(Val::Px(CORNER_RADIUS)),
                                border_radius: BorderRadius::all(Val::Px(6.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.2, 0.8, 0.4)),
                            PlayButton,
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("Start Game"),
                                TextFont {
                                    font_size: FontSize::Px(16.0),
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                });
        });
}
