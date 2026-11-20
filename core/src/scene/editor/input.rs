use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    window::PrimaryWindow,
};

use super::*;

use crate::save::design::{design_matches, upsert};
use crate::save::world::{ContinueWorldEvent, EditorReturnTarget, WorldContext};
use crate::scene::widget::ButtonVisualState;


#[derive(Component)]
pub struct BlockSelectButton(pub String);


#[derive(Component)]
pub struct DeployButton;


#[derive(Component)]
pub struct NavBackButton;


#[derive(Component)]
pub struct NavPlayButton;


#[derive(Component)]
pub struct TabButton(pub EditorTab);


pub fn handle_rotation_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    setting: Res<GameSetting>,
    mut editor: ResMut<EditorState>,
) {
    if keyboard.just_pressed(setting.keys.rotate_ccw) {
        editor.rotation = editor.rotation.ccw();
    }
    if keyboard.just_pressed(setting.keys.rotate_cw) {
        editor.rotation = editor.rotation.cw();
    }
}


pub fn on_block_select(
    click: On<PointerClick>,
    buttons: Query<&BlockSelectButton>,
    mut editor: ResMut<EditorState>,
) {
    let Ok(button) = buttons.get(click.event().entity) else { return; };
    if editor.selected_block.as_deref() == Some(button.0.as_str()) {
        editor.selected_block = None;
    } else {
        editor.selected_block = Some(button.0.clone());
    }
}


pub fn sync_block_selection(
    editor: Res<EditorState>,
    mut buttons: Query<(&BlockSelectButton, &mut ButtonVisualState)>,
) {
    for (button, mut state) in &mut buttons {
        *state = if editor.selected_block.as_deref() == Some(button.0.as_str()) {
            ButtonVisualState::Selected
        } else {
            ButtonVisualState::Normal
        };
    }
}


fn current_design(editor: &EditorState) -> ShipDesign {
    ShipDesign {
        version: DESIGN_SCHEMA_VERSION,
        id: Uuid::now_v7(),
        name: editor.filename.clone(),
        blocks: editor
            .grid
            .blocks()
            .map(|block| block.placement.clone())
            .collect(),
        doctrine: editor.doctrine,
    }
}


pub fn on_deploy(
    click: On<PointerClick>,
    buttons: Query<&DeployButton>,
    mut deploy: ResMut<DeployPayload>,
    editor: Res<EditorState>,
) {
    if !buttons.contains(click.event().entity) {
        return;
    }
    upsert(&mut deploy.fleet, current_design(&editor));
}


pub fn sync_deploy_button(
    deploy: Res<DeployPayload>,
    editor: Res<EditorState>,
    mut buttons: Query<(&DeployButton, &mut ButtonVisualState)>,
) {
    let design = current_design(&editor);
    let is_deployed = deploy
        .fleet
        .iter()
        .any(|current| design_matches(current, &design));

    for (_, mut state) in &mut buttons {
        *state = if is_deployed {
            ButtonVisualState::Disabled
        } else {
            ButtonVisualState::Normal
        };
    }
}


pub fn on_nav_back(
    click: On<PointerClick>,
    buttons: Query<&NavBackButton>,
    target: Res<EditorReturnTarget>,
    mut context: ResMut<WorldContext>,
    mut scene: ResMut<NextState<SceneState>>,
) {
    if buttons.contains(click.event().entity) {
        match *target {
            EditorReturnTarget::Game => {
                context.begin_restore();
                scene.set(SceneState::Game);
            }
            EditorReturnTarget::Home => scene.set(SceneState::Home),
        }
    }
}


pub fn on_nav_play(
    click: On<PointerClick>,
    buttons: Query<&NavPlayButton>,
    mut continues: MessageWriter<ContinueWorldEvent>,
) {
    if buttons.contains(click.event().entity) {
        continues.write(ContinueWorldEvent);
    }
}


pub fn on_tab(
    click: On<PointerClick>,
    buttons: Query<&TabButton>,
    mut editor: ResMut<EditorState>,
) {
    if let Ok(button) = buttons.get(click.event().entity) {
        editor.active_tab = button.0;
    }
}


pub fn process_input(
    mut editor: ResMut<EditorState>,
    window: Single<&Window, With<PrimaryWindow>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut camera: Query<(&Camera, &GlobalTransform, &mut Transform, &mut Projection), With<Camera2d>>,
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    mut mouse_motion_events: MessageReader<MouseMotion>,
) {
    editor.place_req = false;
    editor.delete_req = false;

    let Ok((camera, global_transform, mut transform, mut projection)) = camera.single_mut() else {
        return;
    };

    if let Some(cursor_position) = window.cursor_position() {
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
    field_assets: Option<Res<FieldAssets>>,
) {
    if editor.place_req {
        if let (Some(selected_type), Some(mouse_grid_position)) =
            (editor.selected_block.clone(), editor.hovered_grid_position)
        {
            let Some(def) = registry.get(&selected_type) else {
                warn!("No block definition for {:?}", selected_type);
                return;
            };
            let size = def.get_block_size(editor.rotation);
            let grid_position = mouse_grid_position - (size / 2.0).as_ivec2();

            let is_valid = check_placement_validity(
                &editor.grid,
                &registry,
                selected_type,
                grid_position,
                editor.rotation,
            );

            if !is_valid {
                return;
            }

            let placement = BlockPlacement::new(grid_position, &def.block_type, editor.rotation);
            let base = spawn_placed_block(
                &mut commands,
                ship_base.entity(),
                def,
                &placement,
                field_assets.as_deref(),
            );
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
    if editor.delete_req {
        if let Some(grid_position) = editor.hovered_grid_position {
            remove_block_at(&mut commands, &mut editor, &registry, grid_position);
            events.write(ShipSaveChanged);
        }
    }
}


pub fn sync_tabs(
    editor: Res<EditorState>,
    mut buttons: Query<(&TabButton, &mut ButtonVisualState)>,
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
    for (button, mut state) in &mut buttons {
        *state = if button.0 == editor.active_tab {
            ButtonVisualState::Selected
        } else {
            ButtonVisualState::Normal
        };
    }

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
