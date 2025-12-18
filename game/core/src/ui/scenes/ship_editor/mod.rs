use crate::content::structures::block::PlacedBlock;
use crate::content::structures::block::{
    Block, BlockType, FACE_DOWN, FACE_LEFT, FACE_RIGHT, FACE_UP, GridPosition,
};
use crate::content::structures::ship::{SavedBlock, SavedShip};
use crate::content::structures::{registry::BlockRegistry, ship::ShipGrid};
use crate::save::{load_saved_ships, save_ship};
use crate::{AppState, AppVar, GRID_SIZE};

use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
pub struct ShipEditorPlugin;

impl Plugin for ShipEditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorState>()
            .init_resource::<EditorShip>()
            .init_resource::<AvailableShips>()
            .add_message::<ShipSavesChanged>()
            .add_message::<ShipSaveChanged>()
            .add_systems(
                OnEnter(AppState::ShipEditor),
                (
                    setup_editor_ui,
                    refresh_ship_list.after(setup_editor_ui),
                    load_initial_ship.after(refresh_ship_list),
                ),
            )
            .add_systems(
                Update,
                (
                    handle_block_selection,
                    handle_tab_switching,
                    handle_ship_loading,
                    handle_ship_saving.run_if(on_message::<ShipSaveChanged>),
                    refresh_ship_list.run_if(on_message::<ShipSavesChanged>),
                    process_input, // Replaced handle_grid_input
                    draw_grid,     // New system
                    update_placement_preview,
                    handle_placement,
                    handle_block_deletion,
                    update_block_properties_panel,
                    handle_camera_controls,
                    handle_deploy_button,
                    handle_deploy_button_click,
                    handle_nav_buttons,
                )
                    .chain()
                    .run_if(in_state(AppState::ShipEditor)),
            )
            .add_systems(OnExit(AppState::ShipEditor), cleanup_ship_editor);
    }
}

#[derive(Message)]
struct ShipSavesChanged;

#[derive(Message)]
struct ShipSaveChanged;

// --- Resources ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum EditorTab {
    #[default]
    Blocks,
    Ships,
}

#[derive(Resource, Default)]
struct EditorState {
    active_tab: EditorTab,
    selected_block: Option<BlockType>,
    hovered_grid_pos: Option<IVec2>,
    // Input Abstraction
    place_req: bool,
    delete_req: bool,
    // Drag detection for click-to-pan vs click-to-place
    drag_accumulator: f32,
}

#[derive(Resource, Default)]
struct AvailableShips {
    ships: Vec<SavedShip>,
}

#[derive(Resource, Default)]
struct EditorShip {
    name: String,
    grid: ShipGrid,
}

#[derive(Component)]
struct EditorUiRoot;

#[derive(Component)]
struct PlacementPreview;

#[derive(Component)]
struct TabButton(EditorTab);

#[derive(Component)]
struct BlockListPanel;

#[derive(Component)]
struct ShipListPanel;

#[derive(Component)]
struct LoadShipButton(String);

#[derive(Component)]
struct DeployButton;

#[derive(Component)]
struct BackButton;

#[derive(Component)]
struct PlayButton;

// --- Systems ---

fn setup_editor_ui(
    mut commands: Commands,
    registry: Res<BlockRegistry>,
    asset_server: Res<AssetServer>,
) {
    // Root Node (Full Screen)
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
                                row_gap: Val::Px(8.0),
                                display: Display::Flex, // Initially Visible
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
                                                for layer in &def.layers {
                                                    for visual in &layer.visuals {
                                                        icon_parent.spawn((
                                                            ImageNode {
                                                                image: asset_server
                                                                    .load(&visual.sprite),
                                                                ..default()
                                                            },
                                                            Node {
                                                                position_type:
                                                                    PositionType::Absolute,
                                                                width: Val::Percent(100.0),
                                                                height: Val::Percent(100.0),
                                                                ..default()
                                                            },
                                                            Transform::from_translation(
                                                                visual.offset.extend(visual.z),
                                                            ),
                                                        ));
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

#[derive(Component)]
struct BlockPropertiesText;

fn handle_block_selection(
    mut interaction_query: Query<
        (&Interaction, &BlockType, &mut BackgroundColor, &mut Node),
        (Changed<Interaction>, With<Button>),
    >,
    mut editor_state: ResMut<EditorState>,
) {
    for (interaction, block_type, mut bg_color, mut _node) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                // Toggle selection
                if editor_state.selected_block == Some(*block_type) {
                    editor_state.selected_block = None;
                } else {
                    editor_state.selected_block = Some(*block_type);
                }
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.3, 0.3, 0.3));
            }
            Interaction::None => {
                // Highlight if selected
                if editor_state.selected_block == Some(*block_type) {
                    *bg_color = BackgroundColor(Color::srgb(0.4, 0.4, 0.4));
                } else {
                    *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.2).with_alpha(0.8));
                }
            }
        }
    }
}

fn handle_deploy_button_click(
    mut buttons: Query<(&Interaction, &mut BackgroundColor), With<DeployButton>>,
    mut app_var: ResMut<AppVar>,
    editor_ship: Res<EditorShip>,
    blocks_query: Query<(&Block, &GridPosition)>,
) {
    for (interaction, mut bg_color) in &mut buttons {
        if *interaction == Interaction::Pressed {
            // Deploy Logic
            let mut saved_blocks = Vec::new();

            for (block, grid_pos) in &blocks_query {
                saved_blocks.push(SavedBlock {
                    pos: grid_pos.0,
                    block_type: block.block_type,
                    rotation: block.rotation,
                });
            }

            let saved_ship = SavedShip {
                name: editor_ship.name.clone(),
                blocks: saved_blocks,
            };

            app_var.current_ship = Some(saved_ship);
            info!("Deployed ship: {}", editor_ship.name);
        }

        match interaction {
            Interaction::Pressed => *bg_color = BackgroundColor(Color::srgb(0.1, 0.4, 0.8)),
            Interaction::Hovered => *bg_color = BackgroundColor(Color::srgb(0.3, 0.7, 1.0)),
            _def => {}
        }
    }
}

fn handle_deploy_button(
    mut buttons: Query<&mut BackgroundColor, With<DeployButton>>,
    app_var: ResMut<AppVar>,
    editor_ship: Res<EditorShip>,
) {
    let is_deployed = if let Some(current) = &app_var.current_ship {
        current.name == editor_ship.name
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

fn handle_nav_buttons(
    mut interactions: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            Option<&BackButton>,
            Option<&PlayButton>,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<AppState>>,
) {
    for (interaction, mut bg_color, back, play) in &mut interactions {
        if back.is_none() && play.is_none() {
            continue;
        }

        match *interaction {
            Interaction::Pressed => {
                if back.is_some() {
                    next_state.set(AppState::Home);
                } else if play.is_some() {
                    next_state.set(AppState::Game);
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
fn process_input(
    mut editor_state: ResMut<EditorState>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<Camera2d>>,
    mouse: Res<ButtonInput<MouseButton>>,
    // touches: Res<Touches>, // For future mobile support
) {
    // Reset transient flags
    editor_state.place_req = false;
    editor_state.delete_req = false;

    // Use iter().next() instead of get_single() to avoid potential API issues and be safe
    if let Some(cursor_pos) = window.cursor_position() {
        // Check if cursor is over UI (Left/Right panels)
        // Simplified check assuming fixed widths of 220px (left) and 250px (right)
        let width = window.width();
        if cursor_pos.x < 220.0 || cursor_pos.x > width - 250.0 {
            editor_state.hovered_grid_pos = None;
            return;
        }

        // Convert to world coordinates
        if let Ok(world_pos) = camera.0.viewport_to_world_2d(camera.1, cursor_pos) {
            let grid_x = (world_pos.x / GRID_SIZE).round() as i32;
            let grid_y = (world_pos.y / GRID_SIZE).round() as i32;
            editor_state.hovered_grid_pos = Some(IVec2::new(grid_x, grid_y));

            // Process Actions (Abstraction)
            if mouse.just_pressed(MouseButton::Left) {
                editor_state.drag_accumulator = 0.0;
            }
            if mouse.just_released(MouseButton::Left) {
                if editor_state.drag_accumulator < 5.0 {
                    editor_state.place_req = true;
                }
            }
            if mouse.just_pressed(MouseButton::Right) {
                editor_state.delete_req = true;
            }

            // TODO: Add Touch support here later
            // if touches.any_just_pressed() { ... }
        }
    } else {
        editor_state.hovered_grid_pos = None;
    }
}

/// Draws a visual grid around the center of the world
fn draw_grid(mut gizmos: Gizmos) {
    let grid_lines = 50; // Number of lines in each direction from center
    let grid_color = Color::srgb(0.2, 0.2, 0.2);
    let offset = GRID_SIZE / 2.0;

    // Vertical lines
    for i in -grid_lines..=grid_lines {
        let x = i as f32 * GRID_SIZE + offset;
        gizmos.line_2d(
            Vec2::new(x, -(grid_lines as f32) * GRID_SIZE + offset),
            Vec2::new(x, (grid_lines as f32) * GRID_SIZE + offset),
            grid_color,
        );
    }

    // Horizontal lines
    for i in -grid_lines..=grid_lines {
        let y = i as f32 * GRID_SIZE + offset;
        gizmos.line_2d(
            Vec2::new(-(grid_lines as f32) * GRID_SIZE + offset, y),
            Vec2::new((grid_lines as f32) * GRID_SIZE + offset, y),
            grid_color,
        );
    }
}

fn check_placement_validity(
    grid: &ShipGrid,
    registry: &BlockRegistry,
    block_type: BlockType,
    grid_pos: IVec2,
    _rotation: u8, // Unused for now
    blocks_query: &Query<(&Block, &GridPosition)>,
) -> bool {
    let def = registry.get(&block_type).unwrap();

    // 1. Check for overlap
    for voxel in &def.shape.voxels {
        let target_pos = grid_pos + voxel.pos;
        if grid.blocks.contains_key(&target_pos) {
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
        let my_pos = grid_pos + voxel.pos;
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

            let neighbor_pos = my_pos + dir;
            if let Some(&neighbor_entity) = grid.blocks.get(&neighbor_pos) {
                // Found a neighbor. Check if faces match.
                if let Ok((neighbor_block, neighbor_grid_pos)) = blocks_query.get(neighbor_entity) {
                    if let Some(neighbor_def) = registry.get(&neighbor_block.block_type) {
                        // Find which voxel of the neighbor is at neighbor_pos
                        let neighbor_local_pos = neighbor_pos - neighbor_grid_pos.0;

                        // Find the voxel in neighbor definition
                        if let Some(neighbor_voxel) = neighbor_def
                            .shape
                            .voxels
                            .iter()
                            .find(|v| v.pos == neighbor_local_pos)
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

fn update_placement_preview(
    mut commands: Commands,
    editor_state: Res<EditorState>,
    editor_ship: Res<EditorShip>,
    registry: Res<BlockRegistry>,
    asset_server: Res<AssetServer>,
    preview_query: Query<Entity, With<PlacementPreview>>,
    blocks_query: Query<(&Block, &GridPosition)>, // Added query
) {
    if let (Some(selected_type), Some(mouse_grid_pos)) =
        (editor_state.selected_block, editor_state.hovered_grid_pos)
    {
        let def = registry.get(&selected_type).unwrap();

        // Calculate anchor position to center the block on mouse
        let size = def.shape.size;
        let anchor_offset = (size - IVec2::ONE) / 2;
        let grid_pos = mouse_grid_pos - anchor_offset;

        // Calculate center offset
        let size_vec = def.shape.size.as_vec2();
        let center_offset = size_vec * GRID_SIZE * 0.5;

        let pos = Vec3::new(
            grid_pos.x as f32 * GRID_SIZE,
            grid_pos.y as f32 * GRID_SIZE,
            10.0,
        ) + center_offset.extend(0.0);

        // Check validity
        let is_valid = check_placement_validity(
            &editor_ship.grid,
            &registry,
            selected_type,
            grid_pos,
            0,
            &blocks_query,
        );

        // Despawn existing preview
        for entity in &preview_query {
            commands.entity(entity).despawn();
        }

        let tint = if is_valid {
            Color::WHITE.with_alpha(0.8)
        } else {
            Color::srgb(1.0, 0.0, 0.0).with_alpha(0.8)
        };

        commands
            .spawn((
                Transform::from_translation(pos),
                Visibility::Visible,
                PlacementPreview,
            ))
            .with_children(|parent| {
                for layer in &def.layers {
                    for visual in &layer.visuals {
                        let texture_handle = asset_server.load(&visual.sprite);
                        parent.spawn((
                            Sprite {
                                image: texture_handle,
                                custom_size: Some(size_vec * GRID_SIZE - 2.0),
                                color: tint,
                                ..default()
                            },
                            Transform::from_translation(visual.offset.extend(visual.z)),
                        ));
                    }
                }
            });
    } else {
        // Hide preview if nothing selected or hovered
        for entity in &preview_query {
            commands.entity(entity).despawn();
        }
    }
}

fn handle_placement(
    editor_state: Res<EditorState>,
    registry: Res<BlockRegistry>,
    asset_server: Res<AssetServer>,
    blocks_query: Query<(&Block, &GridPosition)>, // Added query
    mut commands: Commands,
    mut editor_ship: ResMut<EditorShip>,
    mut events: MessageWriter<ShipSaveChanged>,
) {
    // Check abstracted input flag instead of raw mouse input
    if editor_state.place_req {
        if let (Some(selected_type), Some(mouse_grid_pos)) =
            (editor_state.selected_block, editor_state.hovered_grid_pos)
        {
            let def = registry.get(&selected_type).unwrap();
            let size = def.shape.size;
            let anchor_offset = (size - IVec2::ONE) / 2;
            let grid_pos = mouse_grid_pos - anchor_offset;

            let is_valid = check_placement_validity(
                &editor_ship.grid,
                &registry,
                selected_type,
                grid_pos,
                0,
                &blocks_query,
            );

            if is_valid {
                spawn_placed_block(
                    &mut commands,
                    &mut editor_ship,
                    &registry,
                    &asset_server,
                    selected_type,
                    grid_pos,
                );
                events.write(ShipSaveChanged);
            } else {
                info!("Cannot place block here!");
            }
        }
    }
}

fn handle_block_deletion(
    mut commands: Commands,
    editor_state: Res<EditorState>,
    mut editor_ship: ResMut<EditorShip>,
    registry: Res<BlockRegistry>,
    blocks_query: Query<(&Block, &GridPosition)>,
    mut events: MessageWriter<ShipSaveChanged>,
) {
    // Check abstracted input flag instead of raw mouse input
    if editor_state.delete_req {
        if let Some(grid_pos) = editor_state.hovered_grid_pos {
            remove_block_at(
                &mut commands,
                &mut editor_ship,
                &registry,
                &blocks_query,
                grid_pos,
            );
            events.write(ShipSaveChanged);
        }
    }
}

fn update_block_properties_panel(
    editor_state: Res<EditorState>,
    registry: Res<BlockRegistry>,
    mut text_query: Query<&mut Text, With<BlockPropertiesText>>,
) {
    if let Some(mut text) = text_query.iter_mut().next() {
        if let Some(block_type) = editor_state.selected_block {
            if let Some(def) = registry.get(&block_type) {
                **text = format!(
                    "Name: {}\nCost: {}\nMass: {}\nHP: {}\n\n{}",
                    def.name, def.cost, def.mass, def.health, def.description
                );
            }
        } else {
            **text = "Select a block...".to_string();
        }
    }
}

fn cleanup_ship_editor(
    mut commands: Commands,
    ui_root: Query<Entity, With<EditorUiRoot>>,
    preview: Query<Entity, With<PlacementPreview>>,
    placed_blocks: Query<Entity, With<PlacedBlock>>,
) {
    info!("Cleaning up Ship Editor Scene");
    if let Some(e) = ui_root.iter().next() {
        commands.entity(e).despawn();
    }
    for e in &preview {
        commands.entity(e).despawn();
    }
    for e in &placed_blocks {
        commands.entity(e).despawn();
    }

    // Clear resources
    commands.remove_resource::<EditorState>();
    commands.remove_resource::<EditorShip>();
}

fn handle_tab_switching(
    mut interaction_query: Query<
        (&Interaction, &TabButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut editor_state: ResMut<EditorState>,
    mut block_list: Single<&mut Node, (With<BlockListPanel>, Without<ShipListPanel>)>,
    mut ship_list: Single<&mut Node, (With<ShipListPanel>, Without<BlockListPanel>)>,
) {
    for (interaction, tab_btn, mut bg_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                editor_state.active_tab = tab_btn.0;
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.4, 0.4, 0.4));
            }
            Interaction::None => {
                if editor_state.active_tab == tab_btn.0 {
                    *bg_color = BackgroundColor(Color::srgb(0.3, 0.3, 0.3));
                } else {
                    *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.2));
                }
            }
        }
    }

    // Update Visibility
    block_list.display = if editor_state.active_tab == EditorTab::Blocks {
        Display::Flex
    } else {
        Display::None
    };

    ship_list.display = if editor_state.active_tab == EditorTab::Ships {
        Display::Flex
    } else {
        Display::None
    };
}

fn load_initial_ship(
    mut commands: Commands,
    mut editor_ship: ResMut<EditorShip>,
    registry: Res<BlockRegistry>,
    asset_server: Res<AssetServer>,
    placed_blocks: Query<Entity, With<PlacedBlock>>,
) {
    let ship_names = load_saved_ships();

    if ship_names.is_empty() {
        info!("No ships available to load.");
        return;
    }

    perform_load_ship(
        &mut commands,
        &mut editor_ship,
        &registry,
        &asset_server,
        &placed_blocks,
        &ship_names[0].name,
    );

    info!("Load initial ship: {}", &ship_names[0].name);
}

fn refresh_ship_list(
    mut commands: Commands,
    mut available_ships: ResMut<AvailableShips>,
    ship_list_panel: Single<Entity, With<ShipListPanel>>,
) {
    available_ships.ships = load_saved_ships();

    info!("Refreshed ship list: {:?}", available_ships.ships.len());

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
                            LoadShipButton(ship.name.clone()),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new(ship.name.replace(".json", "")),
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

fn handle_ship_saving(
    blocks_query: Query<(&Block, &GridPosition)>,
    mut ship_saved_event: MessageWriter<ShipSavesChanged>,
    state: Res<EditorShip>,
) {
    // Re-validate before saving
    let mut has_core = false;
    let mut block_count = 0;

    for (block, _) in &blocks_query {
        block_count += 1;
        if block.block_type == BlockType::Core {
            has_core = true;
        }
    }

    if !has_core || block_count <= 1 {
        return;
    }

    perform_save_ship(state.name.clone(), &blocks_query, &mut ship_saved_event);
}

fn handle_ship_loading(
    mut commands: Commands,
    mut editor_ship: ResMut<EditorShip>,
    registry: Res<BlockRegistry>,
    asset_server: Res<AssetServer>,
    placed_blocks: Query<Entity, With<PlacedBlock>>,
    mut interaction_query: Query<
        (&Interaction, &LoadShipButton),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, load_btn) in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            perform_load_ship(
                &mut commands,
                &mut editor_ship,
                &registry,
                &asset_server,
                &placed_blocks,
                &load_btn.0,
            );
        }
    }
}
fn spawn_placed_block(
    commands: &mut Commands,
    editor_ship: &mut EditorShip,
    registry: &BlockRegistry,
    asset_server: &AssetServer,
    block_type: BlockType,
    grid_pos: IVec2,
) -> bool {
    if let Some(def) = registry.get(&block_type) {
        let entity = def.render(commands, asset_server, grid_pos, None);

        for voxel in &def.shape.voxels {
            editor_ship.grid.insert(grid_pos + voxel.pos, entity);
        }
    }

    info!("Placed block {:?} at {:?}", block_type, grid_pos);

    return true;
}

fn remove_block_at(
    commands: &mut Commands,
    editor_ship: &mut EditorShip,
    registry: &BlockRegistry,
    blocks_query: &Query<(&Block, &GridPosition)>,
    grid_pos: IVec2,
) {
    if let Some(&entity) = editor_ship.grid.get(grid_pos) {
        // Get the block info to find all occupied cells
        if let Ok((block, grid_pos_comp)) = blocks_query.get(entity) {
            let anchor = grid_pos_comp.0;
            if let Some(def) = registry.get(&block.block_type) {
                // Remove all occupied cells from the grid
                for voxel in &def.shape.voxels {
                    editor_ship.grid.remove(anchor + voxel.pos);
                }

                // Despawn the entity
                commands.entity(entity).despawn();
                info!("Deleted block at {:?}", grid_pos);
            }
        }
    }
}

fn perform_save_ship(
    name: String,
    blocks_query: &Query<(&Block, &GridPosition)>,
    ship_saved_event: &mut MessageWriter<ShipSavesChanged>,
) {
    // 1. Serialize
    let mut saved_blocks = Vec::new();
    // We need to iterate over unique blocks. The grid contains entity refs for every voxel.
    // We can iterate the query instead.

    for (block, grid_pos) in blocks_query {
        saved_blocks.push(SavedBlock {
            pos: grid_pos.0,
            block_type: block.block_type,
            rotation: block.rotation,
        });
    }

    let saved_ship = SavedShip {
        name,
        blocks: saved_blocks,
    };

    if let Err(e) = save_ship(&saved_ship) {
        error!("Fail to save ship: {:?}", e)
    } else {
        ship_saved_event.write(ShipSavesChanged);
        info!("Ship saved at: {}", saved_ship.name);
    }
}

fn perform_load_ship(
    commands: &mut Commands,
    editor_ship: &mut EditorShip,
    registry: &BlockRegistry,
    asset_server: &AssetServer,
    placed_blocks: &Query<Entity, With<PlacedBlock>>,
    ship_name: &str,
) {
    let ships = load_saved_ships();

    if let Some(ship_to_load) = ships.into_iter().find(|ship| ship.name == ship_name) {
        info!("Loading ship: {}", ship_to_load.name);

        // 1. Clear current ship
        editor_ship.grid = ShipGrid::new();
        editor_ship.name = ship_to_load.name;

        for entity in placed_blocks {
            commands.entity(entity).despawn();
        }

        // 2. Spawn new blocks
        for saved_block in ship_to_load.blocks {
            if let Some(def) = registry.get(&saved_block.block_type) {
                let entity = def.render(commands, asset_server, saved_block.pos, None);

                for voxel in &def.shape.voxels {
                    editor_ship.grid.insert(saved_block.pos + voxel.pos, entity);
                }
            }
        }
    } else {
        error!("Ship {} not found", ship_name);
    }
}

fn handle_camera_controls(
    mut query: Query<(&mut Transform, &mut Projection), With<Camera2d>>,
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    mut mouse_motion_events: MessageReader<MouseMotion>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut editor_state: ResMut<EditorState>,
) {
    if let Ok((mut transform, mut projection)) = query.single_mut() {
        // Zoom
        for event in mouse_wheel_events.read() {
            if let Projection::Orthographic(ortho) = &mut *projection {
                let zoom_speed = 0.1;
                ortho.scale -= event.y * zoom_speed * ortho.scale;
                ortho.scale = ortho.scale.clamp(0.1, 5.0);
            }
        }

        // Pan
        if mouse_button_input.pressed(MouseButton::Left) {
            let scale = if let Projection::Orthographic(ortho) = &*projection {
                ortho.scale
            } else {
                1.0
            };

            for event in mouse_motion_events.read() {
                // Pan speed scales with zoom level for consistent feel
                let pan_speed = scale * 0.5;
                transform.translation.x -= event.delta.x * pan_speed;
                transform.translation.y += event.delta.y * pan_speed;
                editor_state.drag_accumulator += event.delta.length();
            }
        }
    }
}
