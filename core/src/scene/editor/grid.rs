use super::*;

pub fn draw_grid(mut gizmos: Gizmos) {
    let grid_lines = 50;
    let grid_color = Color::srgb(0.2, 0.2, 0.2);

    for i in -grid_lines..=grid_lines {
        let x: f32 = i as f32 * GRID_SIZE;
        gizmos.line_2d(
            Vec2::new(x, -(grid_lines as f32) * GRID_SIZE),
            Vec2::new(x, (grid_lines as f32) * GRID_SIZE),
            grid_color,
        );
    }

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
    block_type: String,
    grid_position: IVec2,
    _rotation: u8,
) -> bool {
    let Some(def) = registry.get(&block_type) else {
        warn!("No block definition for {:?}", block_type);
        return false;
    };

    for voxel in &def.shape.voxels {
        let target_position = grid_position + voxel.position;
        if grid.cells.contains_key(&target_position) {
            return false;
        }
    }

    if grid.blocks.is_empty() {
        return true;
    }

    for voxel in &def.shape.voxels {
        let my_position = grid_position + voxel.position;
        let directions = [
            (IVec2::new(1, 0), FACE_RIGHT, FACE_LEFT),
            (IVec2::new(-1, 0), FACE_LEFT, FACE_RIGHT),
            (IVec2::new(0, 1), FACE_UP, FACE_DOWN),
            (IVec2::new(0, -1), FACE_DOWN, FACE_UP),
        ];

        for (dir, my_req_face, neighbor_req_face) in directions {
            if (voxel.connectable_faces & my_req_face) == 0 {
                continue;
            }

            let neighbor_position = my_position + dir;
            if let Some(neighbor_block) = grid.block_at(neighbor_position) {
                if let Some(neighbor_def) = registry.get(&neighbor_block.placement.block_type) {
                    let neighbor_local_position =
                        neighbor_position - neighbor_block.placement.position;

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

    false
}

pub fn update_placement_preview(
    mut commands: Commands,
    editor: Res<EditorState>,
    registry: Res<BlockRegistry>,
    preview: Query<Entity, With<PlacementPreview>>,
) {
    if let (Some(selected_type), Some(mouse_grid_position)) =
        (editor.selected_block.clone(), editor.hovered_grid_position)
    {
        let Some(def) = registry.get(&selected_type) else {
            warn!("No block definition for {:?}", selected_type);
            for entity in &preview {
                commands.entity(entity).despawn();
            }
            return;
        };

        let size = def.shape.size;
        let anchor_offset = size / 2;
        let grid_position = mouse_grid_position - anchor_offset;

        let is_valid =
            check_placement_validity(&editor.grid, &registry, selected_type, grid_position, 0);

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
            .spawn((
                transform,
                Visibility::Visible,
                PlacementPreview,
                DespawnOnExit(SceneState::ShipEditor),
            ))
            .with_children(|icon_parent| {
                for feature in &def.features {
                    match feature {
                        Feature::Texture(visual) => {
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
                        Feature::Turret(visual) => {
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

pub fn update_block_properties_panel(
    editor: Res<EditorState>,
    registry: Res<BlockRegistry>,
    mut text: Query<&mut Text, With<BlockPropertiesText>>,
) {
    if let Some(mut text) = text.iter_mut().next() {
        if let Some(ref block_type) = editor.selected_block {
            if let Some(def) = registry.get(block_type) {
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
