use crate::content::block::base::{
    Block, BlockComponent, BlockDef, BlockType, FACE_DOWN, FACE_LEFT, FACE_RIGHT, FACE_UP,
    GridPosition,
};
use crate::content::block::registry::BlockRegistry;

use crate::content::ship::base::{Ship, ShipGrid};
use crate::content::team::Team;
use crate::save::base::{SchematicBlockData, SchematicData};
use crate::save::io::{load_saved_schematics, save_schematic};
use crate::world::base::{
    get_block_half_size, get_block_size, get_block_transform, render_block_base,
    render_block_component,
};
use crate::{GRID_SIZE, GameState, SceneState};

use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use uuid::Uuid;
pub struct ShipEditorPlugin;

impl Plugin for ShipEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ShipSavesChanged>()
            .add_message::<ShipSaveChanged>()
            .init_resource::<EditorState>()
            .add_systems(
                OnEnter(SceneState::ShipEditor),
                (
                    setup_editor_ui,
                    load_initial_ship.after(setup_editor_ui),
                    refresh_ship_list.after(load_initial_ship),
                ),
            )
            .add_systems(OnExit(SceneState::ShipEditor), cleanup_ship_editor)
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
                    handle_deploy_button,
                    handle_deploy_button_click,
                    handle_nav_buttons,
                )
                    .chain()
                    .run_if(in_state(SceneState::ShipEditor)),
            );
    }
}

#[derive(Message)]
struct ShipSavesChanged;

#[derive(Message)]
struct ShipSaveChanged;

// --- Resources ---

#[derive(Eq, Default, Clone, Copy, PartialEq, Debug)]
enum EditorTab {
    #[default]
    Blocks,
    Ships,
}

#[derive(Resource, Default)]
struct EditorState {
    active_tab: EditorTab,
    selected_block: Option<BlockType>,
    hovered_grid_position: Option<IVec2>,
    // Input Abstraction
    place_req: bool,
    delete_req: bool,
    is_dragging: bool,
    // Drag detection for click-to-pan vs click-to-place
    drag_accumulator: f32,
    filename: String,
    grid: ShipGrid,
    ships: Vec<SchematicData>,
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
struct LoadShipButton {
    filename: String,
}

#[derive(Component)]
struct DeployButton;

#[derive(Component)]
struct BackButton;

#[derive(Component)]
struct PlayButton;

#[derive(Component)]
struct ShipBase;

#[derive(Component)]
struct PlacedBlock;

fn cleanup_ship_editor(
    mut commands: Commands,
    ui_root: Query<Entity, With<EditorUiRoot>>,
    preview: Query<Entity, With<PlacementPreview>>,
    blocks: Query<Entity, With<PlacedBlock>>,
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

fn setup_editor_ui(mut commands: Commands, registry: Res<BlockRegistry>, assets: Res<AssetServer>) {
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
                                                for component in &def.components {
                                                    match component {
                                                        BlockComponent::Sprite(visual) => {
                                                            icon_parent.spawn((
                                                                ImageNode {
                                                                    image: assets
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
                                                            ));
                                                        }
                                                        BlockComponent::Turret(visual) => {
                                                            icon_parent.spawn((
                                                                ImageNode {
                                                                    image: assets
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

#[derive(Component)]
struct BlockPropertiesText;

fn handle_block_selection(
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

fn handle_deploy_button_click(
    mut buttons: Query<(&Interaction, &mut BackgroundColor), With<DeployButton>>,
    mut app_var: ResMut<GameState>,
    editor: Res<EditorState>,
    blocks: Query<(&Block, &GridPosition)>,
) {
    for (interaction, mut bg_color) in &mut buttons {
        if *interaction == Interaction::Pressed {
            // Deploy Logic
            let saved_blocks = blocks
                .iter()
                .map(|(block, grid_position)| SchematicBlockData {
                    id: Uuid::now_v7(),
                    position: grid_position.0,
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
            info!("Deployed ship: {}", editor.filename);
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
fn process_input(
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

            let pan_speed = scale * 0.5;

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
fn draw_grid(mut gizmos: Gizmos) {
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

fn check_placement_validity(
    grid: &ShipGrid,
    registry: &BlockRegistry,
    block_type: BlockType,
    grid_position: IVec2,
    _rotation: u8, // Unused for now
    blocks: &Query<(&Block, &GridPosition)>,
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
                if let Ok((neighbor_block, neighbor_grid_position)) = blocks.get(neighbor_entity) {
                    if let Some(neighbor_def) = registry.get(&neighbor_block.block_type) {
                        // Find which voxel of the neighbor is at neighbor_pos
                        let neighbor_local_position = neighbor_position - neighbor_grid_position.0;

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

fn update_placement_preview(
    mut commands: Commands,
    editor: Res<EditorState>,
    registry: Res<BlockRegistry>,
    assets: Res<AssetServer>,
    preview: Query<Entity, With<PlacementPreview>>,
    blocks: Query<(&Block, &GridPosition)>, // Added query
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

        let half_size = get_block_half_size(def);
        let transform = get_block_transform(grid_position, half_size);

        commands
            .spawn((transform, Visibility::Visible, PlacementPreview))
            .with_children(|icon_parent| {
                for component in &def.components {
                    match component {
                        BlockComponent::Sprite(visual) => {
                            icon_parent.spawn((
                                Sprite {
                                    image: assets.load(&visual.sprite),
                                    custom_size: Some(get_block_size(def)),
                                    color: tint,
                                    ..default()
                                },
                                Transform::from_translation(visual.offset.extend(visual.z)),
                            ));
                        }
                        BlockComponent::Turret(visual) => {
                            icon_parent.spawn((
                                Sprite {
                                    image: assets.load(&visual.sprite),
                                    custom_size: Some(get_block_size(def)),
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

fn handle_placement(
    registry: Res<BlockRegistry>,
    assets: Res<AssetServer>,
    blocks: Query<(&Block, &GridPosition)>, // Added query
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

            let base = spawn_placed_block(
                &mut commands,
                &assets,
                ship_base.entity(),
                def,
                grid_position,
            );

            for voxel in &def.shape.voxels {
                editor.grid.insert(grid_position + voxel.position, base);
            }

            events.write(ShipSaveChanged);
        }
    }
}

fn handle_block_deletion(
    mut commands: Commands,
    mut editor: ResMut<EditorState>,
    registry: Res<BlockRegistry>,
    blocks: Query<(&Block, &GridPosition)>,
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

fn update_block_properties_panel(
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

fn handle_tab_switching(
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

fn load_initial_ship(
    mut commands: Commands,
    mut editor: ResMut<EditorState>,
    registry: Res<BlockRegistry>,
    assets: Res<AssetServer>,
    placed_blocks: Query<Entity, With<PlacedBlock>>,
    ship_base: Single<Entity, With<ShipBase>>,
) {
    info!("Loading initial ship...");
    let ship_names = load_saved_schematics();

    if ship_names.is_empty() {
        info!("No ships available to load.");
        return;
    }

    let filename = &ship_names[0].filename;

    despawn_ship(&mut commands, &placed_blocks);
    spawn_ship(
        &mut commands,
        &mut editor,
        &registry,
        &assets,
        &filename,
        ship_base.entity(),
    );
}

fn refresh_ship_list(
    mut commands: Commands,
    mut available_ships: ResMut<EditorState>,
    ship_list_panel: Single<Entity, With<ShipListPanel>>,
) {
    available_ships.ships = load_saved_schematics();

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

fn handle_ship_saving(
    blocks: Query<(&Block, &GridPosition)>,
    mut ship_saved_event: MessageWriter<ShipSavesChanged>,
    state: Res<EditorState>,
) {
    // Re-validate before saving
    let mut has_core = false;
    let mut block_count = 0;

    for (block, _) in &blocks {
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

fn handle_ship_loading(
    mut commands: Commands,
    mut editor: ResMut<EditorState>,
    registry: Res<BlockRegistry>,
    assets: Res<AssetServer>,
    placed_blocks: Query<Entity, With<PlacedBlock>>,
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
                &assets,
                &load_btn.filename,
                ship_base.entity(),
            );
        }
    }
}
fn spawn_placed_block(
    commands: &mut Commands,
    assets: &AssetServer,
    base: Entity,
    block_def: &BlockDef,
    grid_position: IVec2,
) -> Entity {
    let team = Team::default();
    let base = render_block_base(
        commands,
        base,
        block_def,
        Uuid::now_v7(),
        grid_position,
        team,
    );

    commands.entity(base).insert(PlacedBlock);

    for component in &block_def.components {
        render_block_component(commands, base, assets, block_def, component, team);
    }

    return base;
}

fn remove_block_at(
    commands: &mut Commands,
    editor: &mut EditorState,
    registry: &BlockRegistry,
    blocks: &Query<(&Block, &GridPosition)>,
    grid_position: IVec2,
) {
    if let Some(&entity) = editor.grid.get(grid_position) {
        // Get the block info to find all occupied cells
        if let Ok((block, grid_pos_comp)) = blocks.get(entity) {
            let anchor = grid_pos_comp.0;
            if let Some(def) = registry.get(&block.block_type) {
                // Remove all occupied cells from the grid
                for voxel in &def.shape.voxels {
                    editor.grid.remove(anchor + voxel.position);
                }

                // Despawn the entity
                commands.entity(entity).despawn();
                info!("Deleted block at {:?}", grid_position);
            }
        }
    }
}

fn perform_save_schematic(
    id: Uuid,
    name: String,
    filename: String,
    blocks: &Query<(&Block, &GridPosition)>,
    ship_saved_event: &mut MessageWriter<ShipSavesChanged>,
) {
    let saved_blocks = blocks
        .iter()
        .map(|(block, grid_position)| SchematicBlockData {
            id: Uuid::now_v7(),
            position: grid_position.0,
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
        info!("Ship saved at: {}", ship_data.name);
    }
}

fn despawn_ship(commands: &mut Commands, placed_blocks: &Query<Entity, With<PlacedBlock>>) {
    for entity in placed_blocks {
        commands.entity(entity).despawn();
    }
}

fn spawn_ship(
    commands: &mut Commands,
    editor: &mut EditorState,
    registry: &BlockRegistry,
    assets: &AssetServer,
    filename: &str,
    ship_base: Entity,
) {
    let ships = load_saved_schematics();

    if let Some(ship_to_load) = ships.into_iter().find(|ship| ship.filename == filename) {
        info!("Loading ship: {}", ship_to_load.name);

        editor.grid = ShipGrid::new();
        editor.filename = ship_to_load.filename;

        if ship_to_load.blocks.is_empty() {
            warn!("Ship {} is empty", editor.filename);
            return;
        }

        // 2. Spawn new blocks
        for saved_block in ship_to_load.blocks {
            if let Some(def) = registry.get(&saved_block.block_type) {
                let base = spawn_placed_block(
                    commands,
                    &assets,
                    ship_base.entity(),
                    def,
                    saved_block.position,
                );

                for voxel in &def.shape.voxels {
                    editor
                        .grid
                        .insert(saved_block.position + voxel.position, base);
                }
            }
        }
    } else {
        error!("Ship {} not found", filename);
    }
}
