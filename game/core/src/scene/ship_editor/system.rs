use bevy::{
    input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
    picking::hover::HoverMap,
    prelude::*,
    window::PrimaryWindow,
};
use uuid::Uuid;

use crate::{
    GameState, SceneState,
    constant::GRID_SIZE,
    content::{
        block::{
            component::{
                Block, BlockComponent, BlockDef, BlockType, FACE_DOWN, FACE_LEFT, FACE_RIGHT,
                FACE_UP,
            },
            resource::BlockRegistry,
        },
        ship::component::{Ship, ShipGrid},
    },
    save::{
        component::{SchematicBlockData, SchematicData},
        io::{load_saved_schematics, save_schematic},
    },
    scene::ship_editor::{
        component::*,
        message::{ShipSaveChanged, ShipSavesChanged},
        resource::EditorState,
    },
};

pub fn cleanup_ship_editor(
    mut commands: Commands,
    ui_root: Query<Entity, With<EditorUiRoot>>,
    preview: Query<Entity, With<PlacementPreview>>,
    blocks: Query<Entity, With<Block>>,
    ships: Query<Entity, With<Ship>>,
    ship_bases: Query<Entity, With<ShipBase>>,
) {
    if let Some(e) = ui_root.iter().next() {
        commands.entity(e).despawn();
    }

    for e in &preview {
        commands.entity(e).despawn();
    }

    for e in &blocks {
        commands.entity(e).despawn();
    }

    for e in &ships {
        commands.entity(e).despawn();
    }

    for e in &ship_bases {
        commands.entity(e).despawn();
    }
}

pub fn send_scroll_events(
    mut mouse_wheel_reader: MessageReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    for mouse_wheel in mouse_wheel_reader.read() {
        let mut delta = -Vec2::new(mouse_wheel.x, mouse_wheel.y);

        if mouse_wheel.unit == MouseScrollUnit::Line {
            delta *= 21.0;
        }

        if keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
            std::mem::swap(&mut delta.x, &mut delta.y);
        }

        for pointer_map in hover_map.values() {
            for entity in pointer_map.keys().copied() {
                commands.trigger(Scroll { entity, delta });
            }
        }
    }
}

/// UI scrolling event.
#[derive(EntityEvent, Debug)]
#[entity_event(propagate, auto_propagate)]
pub struct Scroll {
    entity: Entity,
    /// Scroll delta in logical coordinates.
    delta: Vec2,
}

pub fn on_scroll_handler(
    mut scroll: On<Scroll>,
    mut query: Query<(&mut ScrollPosition, &Node, &ComputedNode)>,
) {
    let Ok((mut scroll_position, node, computed)) = query.get_mut(scroll.entity) else {
        return;
    };

    let max_offset = (computed.content_size() - computed.size()) * computed.inverse_scale_factor();

    let delta = &mut scroll.delta;
    if node.overflow.x == OverflowAxis::Scroll && delta.x != 0. {
        // Is this node already scrolled all the way in the direction of the scroll?
        let max = if delta.x > 0. {
            scroll_position.x >= max_offset.x
        } else {
            scroll_position.x <= 0.
        };

        if !max {
            scroll_position.x += delta.x;
            // Consume the X portion of the scroll delta.
            delta.x = 0.;
        }
    }

    if node.overflow.y == OverflowAxis::Scroll && delta.y != 0. {
        // Is this node already scrolled all the way in the direction of the scroll?
        let max = if delta.y > 0. {
            scroll_position.y >= max_offset.y
        } else {
            scroll_position.y <= 0.
        };

        if !max {
            scroll_position.y += delta.y;
            // Consume the Y portion of the scroll delta.
            delta.y = 0.;
        }
    }

    // Stop propagating when the delta is fully consumed.
    if *delta == Vec2::ZERO {
        scroll.propagate(false);
    }
}

pub fn setup_editor_ui(
    mut commands: Commands,
    registry: Res<BlockRegistry>,
    mut camera: Query<&mut Transform, With<Camera2d>>,
) {
    if let Ok(mut transform) = camera.single_mut() {
        transform.translation = Vec3::new(0.0, 0.0, 0.0);
    }
    // Root Node (Full Screen)
    commands.spawn((ShipBase, Visibility::Inherited, Transform::default()));

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            EditorUiRoot,
        ))
        .with_children(|parent| {
            // Left Panel (Block Palette & Ships)
            parent
                .spawn((
                    Node {
                        width: Val::Px(220.0), // Fixed width for better layout
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(10.0)),
                        border: UiRect::right(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
                ))
                .with_children(|left_panel| {
                    // --- Back Button ---
                    left_panel
                        .spawn((
                            Button,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(30.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::bottom(Val::Px(10.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.4, 0.4, 0.4)),
                            BorderRadius::all(Val::Px(6.0)),
                            BackButton,
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("Back to Home"),
                                TextFont {
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });

                    // --- Tab Header ---
                    left_panel
                        .spawn(Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(30.0),
                            margin: UiRect::bottom(Val::Px(10.0)),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        })
                        .with_children(|header| {
                            // Blocks Tab Button
                            header
                                .spawn((
                                    Button,
                                    Node {
                                        width: Val::Percent(48.0),
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
                                            font_size: 16.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                });

                            // Ships Tab Button
                            header
                                .spawn((
                                    Button,
                                    Node {
                                        width: Val::Percent(48.0),
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
                                            font_size: 16.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                });
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
                                        Button,
                                        Node {
                                            width: Val::Percent(100.0),
                                            height: Val::Px(60.0),
                                            justify_content: JustifyContent::Start,
                                            align_items: AlignItems::Center,
                                            border: UiRect::all(Val::Px(1.0)),
                                            padding: UiRect::all(Val::Px(8.0)),
                                            ..default()
                                        },
                                        BorderRadius::all(Val::Px(8.0)),
                                        BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                                        *block_type,
                                    ))
                                    .with_children(|btn| {
                                        // Block Icon/Sprite
                                        // Create a container for the stacked layers
                                        btn.spawn(Node {
                                            width: Val::Px(40.0),
                                            height: Val::Px(40.0),
                                            flex_shrink: 0.0,
                                            ..default()
                                        })
                                        .with_children(
                                            |icon_parent| {
                                                for component in &def.components {
                                                    match component {
                                                        BlockComponent::Sprite(visual) => {
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
                                                        BlockComponent::Turret(visual) => {
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
                                            width: Val::Px(12.0),
                                            ..default()
                                        });

                                        // Block Name
                                        btn.spawn((
                                            Text::new(&def.name),
                                            TextFont {
                                                font_size: 14.0,
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
                        width: Val::Px(250.0), // Fixed width
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(10.0)),
                        border: UiRect::left(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
                ))
                .with_children(|right_panel| {
                    right_panel.spawn((
                        Text::new("Properties"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Node {
                            margin: UiRect::bottom(Val::Px(10.0)),
                            ..default()
                        },
                    ));

                    right_panel.spawn((
                        Text::new("Select a block..."),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                        BlockPropertiesText,
                    ));

                    // Instructions
                    right_panel.spawn((
                        Text::new("\n\nLeft Click: Place\nRight Click: Delete"),
                        TextFont {
                            font_size: 12.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.6, 0.6, 0.6)),
                        Node {
                            margin: UiRect::top(Val::Px(20.0)),
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
                            Button,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(30.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::top(Val::Px(10.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.2, 0.6, 1.0)),
                            BorderRadius::all(Val::Px(6.0)),
                            DeployButton,
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("Deploy"),
                                TextFont {
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });

                    // Play Button
                    right_panel
                        .spawn((
                            Button,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Px(30.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::top(Val::Px(10.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.2, 0.8, 0.4)),
                            BorderRadius::all(Val::Px(6.0)),
                            PlayButton,
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("Start Game"),
                                TextFont {
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                });
        });
}

pub fn handle_block_selection(
    mut interaction: Query<
        (&Interaction, &BlockType, &mut BackgroundColor, &mut Node),
        (Changed<Interaction>, With<Button>),
    >,
    mut editor: ResMut<EditorState>,
) {
    for (interaction, block_type, mut bg_color, mut _node) in &mut interaction {
        match *interaction {
            Interaction::Pressed => {
                // Toggle selection
                if editor.selected_block == Some(*block_type) {
                    editor.selected_block = None;
                } else {
                    editor.selected_block = Some(*block_type);
                }
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.3, 0.3, 0.3));
            }
            Interaction::None => {
                // Highlight if selected
                if editor.selected_block == Some(*block_type) {
                    *bg_color = BackgroundColor(Color::srgb(0.4, 0.4, 0.4));
                } else {
                    *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.2).with_alpha(0.8));
                }
            }
        }
    }
}

pub fn handle_deploy_button_click(
    mut buttons: Query<(&Interaction, &mut BackgroundColor), With<DeployButton>>,
    mut app_var: ResMut<GameState>,
    editor: Res<EditorState>,
    blocks: Query<&Block>,
) {
    for (interaction, mut bg_color) in &mut buttons {
        if *interaction == Interaction::Pressed {
            // Deploy Logic
            let saved_blocks = blocks
                .iter()
                .map(|block| SchematicBlockData {
                    id: Uuid::now_v7(),
                    position: block.position,
                    block_type: block.block_type,
                    rotation: block.rotation,
                })
                .collect::<Vec<SchematicBlockData>>();

            let ship_data = SchematicData {
                id: Uuid::now_v7(),
                name: editor.filename.clone(),
                filename: editor.filename.clone(),
                blocks: saved_blocks,
            };

            app_var.current_ship = Some(ship_data);
        }

        match interaction {
            Interaction::Pressed => *bg_color = BackgroundColor(Color::srgb(0.1, 0.4, 0.8)),
            Interaction::Hovered => *bg_color = BackgroundColor(Color::srgb(0.3, 0.7, 1.0)),
            _def => {}
        }
    }
}

pub fn handle_deploy_button(
    mut buttons: Query<&mut BackgroundColor, With<DeployButton>>,
    app_var: ResMut<GameState>,
    editor: Res<EditorState>,
) {
    let is_deployed = if let Some(current) = &app_var.current_ship {
        current.filename == editor.filename
    } else {
        false
    };

    for mut bg_color in &mut buttons {
        if is_deployed {
            *bg_color = BackgroundColor(Color::srgb(0.3, 0.3, 0.3));
        } else {
            *bg_color = BackgroundColor(Color::srgb(0.2, 0.6, 1.0));
        }
    }
}

pub fn handle_nav_buttons(
    mut interactions: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            Option<&BackButton>,
            Option<&PlayButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<SceneState>>,
) {
    for (interaction, mut bg_color, back, play) in &mut interactions {
        if back.is_none() && play.is_none() {
            continue;
        }

        match *interaction {
            Interaction::Pressed => {
                if back.is_some() {
                    next_state.set(SceneState::Home);
                } else if play.is_some() {
                    next_state.set(SceneState::Game);
                }
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.5, 0.5, 0.5));
                if play.is_some() {
                    *bg_color = BackgroundColor(Color::srgb(0.3, 0.9, 0.5));
                }
            }
            Interaction::None => {
                if back.is_some() {
                    *bg_color = BackgroundColor(Color::srgb(0.4, 0.4, 0.4));
                } else if play.is_some() {
                    *bg_color = BackgroundColor(Color::srgb(0.2, 0.8, 0.4));
                }
            }
        }
    }
}

/// Abstract input processing system
/// Converts raw inputs (Mouse, Touch) into high-level editor intents.
pub fn process_input(
    mut editor: ResMut<EditorState>,
    window: Single<&Window, With<PrimaryWindow>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut camera: Query<(&Camera, &GlobalTransform, &mut Transform, &mut Projection), With<Camera2d>>,
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    mut mouse_motion_events: MessageReader<MouseMotion>,
    // touches: Res<Touches>, // For future mobile support
) {
    // Reset transient flags
    editor.place_req = false;
    editor.delete_req = false;

    let Ok((camera, global_transform, mut transform, mut projection)) = camera.single_mut() else {
        return;
    };

    // Use iter().next() instead of get_single() to avoid potential API issues and be safe
    if let Some(cursor_position) = window.cursor_position() {
        // Check if cursor is over UI (Left/Right panels)
        // Simplified check assuming fixed widths of 220px (left) and 250px (right)
        let width = window.width();
        if cursor_position.x < 220.0 || cursor_position.x > width - 250.0 {
            editor.hovered_grid_position = None;
            return;
        }

        if mouse.just_pressed(MouseButton::Left) {
            editor.drag_accumulator = 0.0;
            editor.is_dragging = false;
            mouse_motion_events.clear();
        }

        if mouse.pressed(MouseButton::Left) {
            let scale = if let Projection::Orthographic(ortho) = &*projection {
                ortho.scale
            } else {
                1.0
            };

            let pan_speed: f32 = scale * 0.5;

            for event in mouse_motion_events.read() {
                transform.translation.x -= event.delta.x * pan_speed;
                transform.translation.y += event.delta.y * pan_speed;

                editor.drag_accumulator += event.delta.length();

                if editor.drag_accumulator > 4.0 {
                    editor.is_dragging = true;
                }
            }
        }

        if mouse.just_released(MouseButton::Left) && !editor.is_dragging {
            editor.place_req = true;
        }

        if mouse.pressed(MouseButton::Right) {
            editor.delete_req = true;
        }

        for event in mouse_wheel_events.read() {
            if let Projection::Orthographic(ortho) = &mut *projection {
                let zoom_speed = 0.1;
                ortho.scale -= event.y * zoom_speed * ortho.scale;
                ortho.scale = ortho.scale.clamp(0.1, 5.0);
            }
        }

        // Convert to world coordinates
        if let Ok(world_position) = camera.viewport_to_world_2d(global_transform, cursor_position) {
            let grid_x = ((world_position.x - GRID_SIZE / 2.0) / GRID_SIZE).round() as i32;
            let grid_y = ((world_position.y - GRID_SIZE / 2.0) / GRID_SIZE).round() as i32;

            editor.hovered_grid_position = Some(IVec2::new(grid_x, grid_y));
        }
    } else {
        editor.hovered_grid_position = None;
    }
}

/// Draws a visual grid around the center of the world
pub fn draw_grid(mut gizmos: Gizmos) {
    let grid_lines = 50; // Number of lines in each direction from center
    let grid_color = Color::srgb(0.2, 0.2, 0.2);

    // Vertical lines
    for i in -grid_lines..=grid_lines {
        let x: f32 = i as f32 * GRID_SIZE;
        gizmos.line_2d(
            Vec2::new(x, -(grid_lines as f32) * GRID_SIZE),
            Vec2::new(x, (grid_lines as f32) * GRID_SIZE),
            grid_color,
        );
    }

    // Horizontal lines
    for i in -grid_lines..=grid_lines {
        let y = i as f32 * GRID_SIZE;
        gizmos.line_2d(
            Vec2::new(-(grid_lines as f32) * GRID_SIZE, y),
            Vec2::new((grid_lines as f32) * GRID_SIZE, y),
            grid_color,
        );
    }
}

pub fn check_placement_validity(
    grid: &ShipGrid,
    registry: &BlockRegistry,
    block_type: BlockType,
    grid_position: IVec2,
    _rotation: u8, // Unused for now
    blocks: &Query<&Block>,
) -> bool {
    let def = registry.get(&block_type).unwrap();

    // 1. Check for overlap
    for voxel in &def.shape.voxels {
        let target_position = grid_position + voxel.position;
        if grid.blocks.contains_key(&target_position) {
            return false;
        }
    }

    // 2. If grid is empty, allow placement (first block)
    if grid.blocks.is_empty() {
        return true;
    }

    // 3. Check connectivity with neighbors
    // We check if ANY of our voxels connects to a valid neighbor.
    for voxel in &def.shape.voxels {
        let my_position = grid_position + voxel.position;
        let directions = [
            (IVec2::new(1, 0), FACE_RIGHT, FACE_LEFT), // Neighbor is Right, I need FACE_RIGHT, Neighbor needs FACE_LEFT
            (IVec2::new(-1, 0), FACE_LEFT, FACE_RIGHT), // Neighbor is Left, I need FACE_LEFT, Neighbor needs FACE_RIGHT
            (IVec2::new(0, 1), FACE_UP, FACE_DOWN), // Neighbor is Up, I need FACE_UP, Neighbor needs FACE_DOWN
            (IVec2::new(0, -1), FACE_DOWN, FACE_UP), // Neighbor is Down, I need FACE_DOWN, Neighbor needs FACE_UP
        ];

        for (dir, my_req_face, neighbor_req_face) in directions {
            // Check if this voxel has the required face
            if (voxel.connectable_faces & my_req_face) == 0 {
                continue;
            }

            let neighbor_position = my_position + dir;
            if let Some(&neighbor_entity) = grid.blocks.get(&neighbor_position) {
                // Found a neighbor. Check if faces match.
                if let Ok(neighbor_block) = blocks.get(neighbor_entity) {
                    if let Some(neighbor_def) = registry.get(&neighbor_block.block_type) {
                        // Find which voxel of the neighbor is at neighbor_pos
                        let neighbor_local_position = neighbor_position - neighbor_block.position;

                        // Find the voxel in neighbor definition
                        if let Some(neighbor_voxel) = neighbor_def
                            .shape
                            .voxels
                            .iter()
                            .find(|v| v.position == neighbor_local_position)
                        {
                            if (neighbor_voxel.connectable_faces & neighbor_req_face) != 0 {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }

    false
}

pub fn update_placement_preview(
    mut commands: Commands,
    editor: Res<EditorState>,
    registry: Res<BlockRegistry>,
    preview: Query<Entity, With<PlacementPreview>>,
    blocks: Query<&Block>, // Added query
) {
    if let (Some(selected_type), Some(mouse_grid_position)) =
        (editor.selected_block, editor.hovered_grid_position)
    {
        let def = registry.get(&selected_type).unwrap();

        // Calculate anchor position to center the block on mouse
        let size = def.shape.size;
        let anchor_offset = size / 2;
        let grid_position = mouse_grid_position - anchor_offset;

        // Check validity
        let is_valid = check_placement_validity(
            &editor.grid,
            &registry,
            selected_type,
            grid_position,
            0,
            &blocks,
        );

        // Despawn existing preview
        for entity in &preview {
            commands.entity(entity).despawn();
        }

        let tint = if is_valid {
            Color::WHITE.with_alpha(0.6)
        } else {
            Color::srgb(1.05, 0.0, 0.0).with_alpha(0.8)
        };

        let transform = def.to_transform(grid_position);

        commands
            .spawn((transform, Visibility::Visible, PlacementPreview))
            .with_children(|icon_parent| {
                for component in &def.components {
                    match component {
                        BlockComponent::Sprite(visual) => {
                            icon_parent.spawn((
                                Sprite {
                                    image: visual.image.clone(),
                                    custom_size: Some(def.get_pixel_size()),
                                    color: tint,
                                    ..default()
                                },
                                Transform::from_translation(visual.offset.extend(visual.z)),
                            ));
                        }
                        BlockComponent::Turret(visual) => {
                            icon_parent.spawn((
                                Sprite {
                                    image: visual.image.clone(),
                                    custom_size: Some(def.get_pixel_size()),
                                    color: tint,
                                    ..default()
                                },
                                Transform::from_translation(visual.offset.extend(visual.z)),
                            ));
                        }
                        _ => {}
                    }
                }
            });
    } else {
        for entity in &preview {
            commands.entity(entity).despawn();
        }
    }
}

pub fn handle_placement(
    registry: Res<BlockRegistry>,
    blocks: Query<&Block>, // Added query
    mut commands: Commands,
    mut editor: ResMut<EditorState>,
    mut events: MessageWriter<ShipSaveChanged>,
    ship_base: Single<Entity, With<ShipBase>>,
) {
    if editor.place_req {
        if let (Some(selected_type), Some(mouse_grid_position)) =
            (editor.selected_block, editor.hovered_grid_position)
        {
            let def = registry.get(&selected_type).unwrap();
            let size = def.shape.size;
            let anchor_offset = size / 2;
            let grid_position = mouse_grid_position - anchor_offset;

            let is_valid = check_placement_validity(
                &editor.grid,
                &registry,
                selected_type,
                grid_position,
                0,
                &blocks,
            );

            if !is_valid {
                return;
            }

            let base = spawn_placed_block(&mut commands, ship_base.entity(), def, grid_position);

            for voxel in &def.shape.voxels {
                editor.grid.insert(grid_position + voxel.position, base);
            }

            events.write(ShipSaveChanged);
        }
    }
}

pub fn handle_block_deletion(
    mut commands: Commands,
    mut editor: ResMut<EditorState>,
    registry: Res<BlockRegistry>,
    blocks: Query<&Block>,
    mut events: MessageWriter<ShipSaveChanged>,
) {
    // Check abstracted input flag instead of raw mouse input
    if editor.delete_req {
        if let Some(grid_position) = editor.hovered_grid_position {
            remove_block_at(
                &mut commands,
                &mut editor,
                &registry,
                &blocks,
                grid_position,
            );
            events.write(ShipSaveChanged);
        }
    }
}

pub fn update_block_properties_panel(
    editor: Res<EditorState>,
    registry: Res<BlockRegistry>,
    mut text: Query<&mut Text, With<BlockPropertiesText>>,
) {
    if let Some(mut text) = text.iter_mut().next() {
        if let Some(block_type) = editor.selected_block {
            if let Some(def) = registry.get(&block_type) {
                **text = format!(
                    "Name: {}\nCost: {}\nMass: {}\nHP: {}\n",
                    def.name, def.cost, def.mass, def.health
                );
            }
        } else {
            **text = "Select a block...".to_string();
        }
    }
}

pub fn handle_tab_switching(
    mut interaction: Query<
        (&Interaction, &TabButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut editor: ResMut<EditorState>,
    mut block_list: Single<&mut Node, (With<BlockListPanel>, Without<ShipListPanel>)>,
    mut ship_list: Single<&mut Node, (With<ShipListPanel>, Without<BlockListPanel>)>,
) {
    for (interaction, tab_btn, mut bg_color) in &mut interaction {
        match *interaction {
            Interaction::Pressed => {
                editor.active_tab = tab_btn.0;
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.4, 0.4, 0.4));
            }
            Interaction::None => {
                if editor.active_tab == tab_btn.0 {
                    *bg_color = BackgroundColor(Color::srgb(0.3, 0.3, 0.3));
                } else {
                    *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.2));
                }
            }
        }
    }

    // Update Visibility
    block_list.display = if editor.active_tab == EditorTab::Blocks {
        Display::Flex
    } else {
        Display::None
    };

    ship_list.display = if editor.active_tab == EditorTab::Ships {
        Display::Flex
    } else {
        Display::None
    };
}

pub fn load_initial_ship(
    mut commands: Commands,
    mut editor: ResMut<EditorState>,
    registry: Res<BlockRegistry>,
    placed_blocks: Query<Entity, With<Block>>,
    ship_base: Single<Entity, With<ShipBase>>,
) {
    let ship_names = load_saved_schematics();

    if ship_names.is_empty() {
        return;
    }

    let filename = &ship_names[0].filename;

    despawn_ship(&mut commands, &placed_blocks);

    spawn_ship(
        &mut commands,
        &mut editor,
        &registry,
        &filename,
        ship_base.entity(),
    );
}

pub fn refresh_ship_list(
    mut commands: Commands,
    mut available_ships: ResMut<EditorState>,
    ship_list_panel: Single<Entity, With<ShipListPanel>>,
) {
    available_ships.ships = load_saved_schematics();

    // 2. Rebuild UI
    commands.entity(ship_list_panel.entity()).despawn_children();

    commands
        .entity(ship_list_panel.entity())
        .with_children(|parent| {
            for ship in &available_ships.ships {
                parent
                    .spawn((Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        margin: UiRect::bottom(Val::Px(4.0)),
                        ..default()
                    },))
                    .with_children(|row| {
                        // Load Button
                        row.spawn((
                            Button,
                            Node {
                                flex_grow: 1.0,
                                height: Val::Percent(100.0),
                                justify_content: JustifyContent::Start,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(1.0)),
                                padding: UiRect::all(Val::Px(8.0)),
                                margin: UiRect::right(Val::Px(4.0)),
                                ..default()
                            },
                            BorderRadius::all(Val::Px(4.0)),
                            BackgroundColor(Color::srgb(0.25, 0.25, 0.25)),
                            LoadShipButton {
                                filename: ship.filename.clone(),
                            },
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new(if ship.name.is_empty() {
                                    ship.filename.clone()
                                } else {
                                    ship.name.clone()
                                }),
                                TextFont {
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                    });
            }
        });
}

pub fn handle_ship_saving(
    blocks: Query<&Block>,
    mut ship_saved_event: MessageWriter<ShipSavesChanged>,
    state: Res<EditorState>,
) {
    // Re-validate before saving
    let mut has_core = false;
    let mut block_count = 0;

    for block in &blocks {
        block_count += 1;
        if block.block_type == BlockType::Core {
            has_core = true;
        }
    }

    if !has_core || block_count <= 1 {
        return;
    }

    perform_save_schematic(
        Uuid::now_v7(),
        state.filename.clone(),
        state.filename.clone(),
        &blocks,
        &mut ship_saved_event,
    );
}

pub fn handle_ship_loading(
    mut commands: Commands,
    mut editor: ResMut<EditorState>,
    registry: Res<BlockRegistry>,
    placed_blocks: Query<Entity, With<Block>>,
    mut interaction: Query<(&Interaction, &LoadShipButton), (Changed<Interaction>, With<Button>)>,
    ship_base: Single<Entity, With<ShipBase>>,
) {
    for (interaction, load_btn) in &mut interaction {
        if *interaction == Interaction::Pressed {
            despawn_ship(&mut commands, &placed_blocks);
            spawn_ship(
                &mut commands,
                &mut editor,
                &registry,
                &load_btn.filename,
                ship_base.entity(),
            );
        }
    }
}
pub fn spawn_placed_block(
    commands: &mut Commands,
    base: Entity,
    def: &BlockDef,
    position: IVec2,
) -> Entity {
    let transform = def.to_transform(position);

    let block = commands
        .spawn((
            Block {
                id: Uuid::now_v7(),
                rotation: 0,
                position,
                block_type: def.block_type,
            },
            Visibility::Visible,
            transform,
        ))
        .id();

    commands.entity(base).add_child(block);

    for component in &def.components {
        let c = commands
            .spawn((Transform::default(), Visibility::Inherited))
            .id();

        commands.entity(block).add_child(c);

        match component {
            BlockComponent::Sprite(visual) => {
                commands.entity(c).insert((
                    Sprite {
                        image: visual.image.clone(),
                        custom_size: Some(def.get_pixel_size()),
                        ..default()
                    },
                    Transform::from_translation(visual.offset.extend(visual.z)),
                ));
            }
            BlockComponent::Turret(visual) => {
                commands.entity(c).insert((
                    Sprite {
                        image: visual.image.clone(),
                        custom_size: Some(def.get_pixel_size()),
                        ..default()
                    },
                    Transform::from_translation(visual.offset.extend(visual.z)),
                ));
            }
            _ => {}
        }
    }

    return block;
}

pub fn remove_block_at(
    commands: &mut Commands,
    editor: &mut EditorState,
    registry: &BlockRegistry,
    blocks: &Query<&Block>,
    grid_position: IVec2,
) {
    let Some(&entity) = editor.grid.get(grid_position) else {
        info!("No block at position {:?}", grid_position);
        return;
    };

    let Ok(block) = blocks.get(entity) else {
        info!("No block entity {:?}", entity);
        return;
    };
    let anchor = block.position;

    let Some(def) = registry.get(&block.block_type) else {
        info!("No block def {:?}", block.block_type);
        return;
    };

    for voxel in &def.shape.voxels {
        editor.grid.remove(anchor + voxel.position);
    }

    commands.entity(entity).despawn();
}

pub fn perform_save_schematic(
    id: Uuid,
    name: String,
    filename: String,
    blocks: &Query<&Block>,
    ship_saved_event: &mut MessageWriter<ShipSavesChanged>,
) {
    let saved_blocks = blocks
        .iter()
        .map(|block| SchematicBlockData {
            id: Uuid::now_v7(),
            position: block.position,
            block_type: block.block_type,
            rotation: block.rotation,
        })
        .collect::<Vec<SchematicBlockData>>();

    let ship_data = SchematicData {
        id,
        filename,
        name,
        blocks: saved_blocks,
    };

    if let Err(e) = save_schematic(&ship_data) {
        error!("Fail to save ship: {:?}", e)
    } else {
        ship_saved_event.write(ShipSavesChanged);
    }
}

pub fn despawn_ship(commands: &mut Commands, placed_blocks: &Query<Entity, With<Block>>) {
    for entity in placed_blocks {
        commands.entity(entity).despawn();
    }
}

pub fn spawn_ship(
    commands: &mut Commands,
    editor: &mut EditorState,
    registry: &BlockRegistry,
    filename: &str,
    ship_base: Entity,
) {
    let ships = load_saved_schematics();

    editor.grid = ShipGrid::new();
    editor.filename = filename.to_string();

    if let Some(ship_to_load) = ships.into_iter().find(|ship| ship.filename == filename) {
        editor.filename = ship_to_load.filename;

        if ship_to_load.blocks.is_empty() {
            warn!("Ship {} is empty", editor.filename);
            return;
        }

        // 2. Spawn new blocks
        for saved_block in ship_to_load.blocks {
            if let Some(def) = registry.get(&saved_block.block_type) {
                let base = spawn_placed_block(commands, ship_base, def, saved_block.position);

                for voxel in &def.shape.voxels {
                    editor
                        .grid
                        .insert(saved_block.position + voxel.position, base);
                }
            }
        }
    } else {
        error!("Ship {} data not found", filename);
    }
}
