use super::*;

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
            mem::swap(&mut delta.x, &mut delta.y);
        }

        for pointer_map in hover_map.values() {
            for entity in pointer_map.keys().copied() {
                commands.trigger(Scroll { entity, delta });
            }
        }
    }
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

pub fn handle_block_selection(
    mut interaction: Query<
        (
            &PickingInteraction,
            &BlockSelection,
            &mut BackgroundColor,
            &mut Node,
        ),
        Changed<PickingInteraction>,
    >,
    mut editor: ResMut<EditorState>,
) {
    for (interaction, block_type, mut bg_color, mut _node) in &mut interaction {
        match *interaction {
            PickingInteraction::Pressed => {
                // Toggle selection
                if editor.selected_block == Some(block_type.0.clone()) {
                    editor.selected_block = None;
                } else {
                    editor.selected_block = Some(block_type.0.clone());
                }
            }
            PickingInteraction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.3, 0.3, 0.3));
            }
            PickingInteraction::None => {
                // Highlight if selected
                if editor.selected_block == Some(block_type.0.clone()) {
                    *bg_color = BackgroundColor(Color::srgb(0.4, 0.4, 0.4));
                } else {
                    *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.2).with_alpha(0.8));
                }
            }
        }
    }
}

pub fn handle_deploy_button_click(
    mut buttons: Query<(&PickingInteraction, &mut BackgroundColor), With<DeployButton>>,
    mut app_var: ResMut<DeployPayload>,
    editor: Res<EditorState>,
) {
    for (interaction, mut bg_color) in &mut buttons {
        if *interaction == PickingInteraction::Pressed {
            // Deploy Logic
            let saved_blocks: Vec<BlockPlacement> = editor
                .grid
                .blocks()
                .map(|block| block.placement.clone())
                .collect();

            app_var.current_ship = Some(ShipDesign {
                version: 1,
                id: Uuid::now_v7(),
                name: editor.filename.clone(),
                blocks: saved_blocks,
                ai: editor.ai_profile.clone(),
            });
        }

        match interaction {
            PickingInteraction::Pressed => *bg_color = BackgroundColor(Color::srgb(0.1, 0.4, 0.8)),
            PickingInteraction::Hovered => *bg_color = BackgroundColor(Color::srgb(0.3, 0.7, 1.0)),
            _def => {}
        }
    }
}

pub fn handle_deploy_button(
    mut buttons: Query<&mut BackgroundColor, With<DeployButton>>,
    app_var: ResMut<DeployPayload>,
    editor: Res<EditorState>,
) {
    let is_deployed = if let Some(current) = &app_var.current_ship {
        current.name == editor.filename
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
            &PickingInteraction,
            &mut BackgroundColor,
            Option<&BackButton>,
            Option<&PlayButton>,
        ),
        Changed<PickingInteraction>,
    >,
    mut next_state: ResMut<NextState<SceneState>>,
) {
    for (interaction, mut bg_color, back, play) in &mut interactions {
        if back.is_none() && play.is_none() {
            continue;
        }

        match *interaction {
            PickingInteraction::Pressed => {
                if back.is_some() {
                    next_state.set(SceneState::Home);
                } else if play.is_some() {
                    next_state.set(SceneState::Game);
                }
            }
            PickingInteraction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.5, 0.5, 0.5));
                if play.is_some() {
                    *bg_color = BackgroundColor(Color::srgb(0.3, 0.9, 0.5));
                }
            }
            PickingInteraction::None => {
                if back.is_some() {
                    *bg_color = BackgroundColor(Color::srgb(0.4, 0.4, 0.4));
                } else if play.is_some() {
                    *bg_color = BackgroundColor(Color::srgb(0.2, 0.8, 0.4));
                }
            }
        }
    }
}

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

pub fn handle_placement(
    registry: Res<BlockRegistry>,
    mut commands: Commands,
    mut editor: ResMut<EditorState>,
    mut events: MessageWriter<ShipSaveChanged>,
    ship_base: Single<Entity, With<ShipBase>>,
) {
    if editor.place_req {
        if let (Some(selected_type), Some(mouse_grid_position)) =
            (editor.selected_block.clone(), editor.hovered_grid_position)
        {
            let Some(def) = registry.get(&selected_type) else {
                warn!("No block definition for {:?}", selected_type);
                return;
            };
            let size = def.shape.size;
            let anchor_offset = size / 2;
            let grid_position = mouse_grid_position - anchor_offset;

            let is_valid =
                check_placement_validity(&editor.grid, &registry, selected_type, grid_position, 0);

            if !is_valid {
                return;
            }

            let placement = BlockPlacement::new(grid_position, &def.block_type, 0);
            let base = spawn_placed_block(&mut commands, ship_base.entity(), def, &placement);
            editor.grid.insert_block(placement, base, def);

            events.write(ShipSaveChanged);
        }
    }
}

pub fn handle_block_deletion(
    mut commands: Commands,
    mut editor: ResMut<EditorState>,
    registry: Res<BlockRegistry>,
    mut events: MessageWriter<ShipSaveChanged>,
) {
    // Check abstracted input flag instead of raw mouse input
    if editor.delete_req {
        if let Some(grid_position) = editor.hovered_grid_position {
            remove_block_at(&mut commands, &mut editor, &registry, grid_position);
            events.write(ShipSaveChanged);
        }
    }
}

pub fn handle_tab_switching(
    mut interaction: Query<
        (&PickingInteraction, &TabButton, &mut BackgroundColor),
        Changed<PickingInteraction>,
    >,
    mut editor: ResMut<EditorState>,
    mut block_list: Single<
        &mut Node,
        (
            With<BlockListPanel>,
            Without<ShipListPanel>,
            Without<AiPanel>,
        ),
    >,
    mut ship_list: Single<
        &mut Node,
        (
            With<ShipListPanel>,
            Without<BlockListPanel>,
            Without<AiPanel>,
        ),
    >,
    mut ai_panel: Single<
        &mut Node,
        (
            With<AiPanel>,
            Without<BlockListPanel>,
            Without<ShipListPanel>,
        ),
    >,
) {
    for (interaction, tab_btn, mut bg_color) in &mut interaction {
        match *interaction {
            PickingInteraction::Pressed => {
                editor.active_tab = tab_btn.0;
            }
            PickingInteraction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.4, 0.4, 0.4));
            }
            PickingInteraction::None => {
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
    ai_panel.display = if editor.active_tab == EditorTab::Ai {
        Display::Flex
    } else {
        Display::None
    };
}
