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
    grid: &BlockGrid,
    registry: &BlockRegistry,
    block_type: String,
    grid_position: IVec2,
    rotation: Rot90,
) -> bool {
    let Some(def) = registry.get(&block_type) else {
        warn!("No block definition for {:?}", block_type);
        return false;
    };

    let footprint = def.shape.get(rotation).0;

    for offset in footprint.cells() {
        if grid.cells.contains_key(&(grid_position + *offset)) {
            return false;
        }
    }

    if grid.blocks.is_empty() {
        return true;
    }

    for offset in footprint.cells() {
        let cell = grid_position + *offset;
        for direction in [IVec2::X, IVec2::NEG_X, IVec2::Y, IVec2::NEG_Y] {
            let Some(neighbor) = grid.block_at(cell + direction) else {
                continue;
            };
            let Some(neighbor_def) = registry.get_symbol(&neighbor.placement.block_type) else {
                continue;
            };
            let neighbor_offset = cell + direction - neighbor.placement.position;
            if def.solid_edge(rotation, *offset, direction)
                && neighbor_def.solid_edge(neighbor.placement.rotation, neighbor_offset, -direction)
            {
                return true;
            }
        }
    }

    false
}

pub fn update_placement_preview(
    mut commands: Commands,
    editor: Res<EditorState>,
    registry: Res<BlockRegistry>,
    mut state: Local<PreviewState>,
    preview: Query<Entity, With<PlacementPreview>>,
    mut sprites: Query<&mut Sprite, With<PreviewSprite>>,
) {
    let key = PreviewKey {
        block: editor.selected_block.clone(),
        hovered: editor.hovered_grid_position,
        rotation: editor.rotation,
        grid_len: editor.grid.blocks.len(),
    };
    
    if key == state.key {
        return;
    }

    state.key = key.clone();

    let current = preview.iter().next();
    let (Some(selected_type), Some(mouse_grid_position)) = (&key.block, key.hovered) else {
        if let Some(entity) = current {
            commands.entity(entity).despawn();
        }
        return;
    };

    let Some(def) = registry.get(selected_type) else {
        warn!("No block definition for {:?}", selected_type);
        if let Some(entity) = current {
            commands.entity(entity).despawn();
        }
        return;
    };

    let size = def.get_block_size(key.rotation);
    let grid_position = mouse_grid_position - (size / 2.0).as_ivec2();
    let is_valid = check_placement_validity(
        &editor.grid,
        &registry,
        selected_type.clone(),
        grid_position,
        key.rotation,
    );
    let tint = if is_valid {
        Color::WHITE.with_alpha(0.6)
    } else {
        Color::srgb(1.05, 0.0, 0.0).with_alpha(0.8)
    };
    let transform = def.to_transform(grid_position, key.rotation);

    let entity = if let Some(entity) = current {
        commands.entity(entity).insert(transform);
        entity
    } else {
        commands
            .spawn((
                transform,
                Visibility::Visible,
                PlacementPreview,
                DespawnOnExit(SceneState::ShipEditor),
            ))
            .id()
    };

    if state.built_for.as_deref() != Some(selected_type.as_str()) {
        commands.entity(entity).despawn_children();
        commands.entity(entity).with_children(|root| {
            root.spawn((visual_transform(def.shape.xform()), Visual)).with_children(|visual| {
                for feature in &def.features {
                    let (image, offset, z) = match feature {
                        Feature::Texture(texture) => {
                            (&texture.image, texture.offset, texture.z)
                        }
                        Feature::Turret(turret) => (&turret.image, turret.offset, turret.z),
                        _ => continue,
                    };
                    visual.spawn((
                        Sprite {
                            image: image.clone(),
                            custom_size: Some(def.get_base_pixel_size()),
                            color: tint,
                            ..default()
                        },
                        Transform::from_translation(offset.extend(z)),
                        PreviewSprite,
                    ));
                }
            });
        });
        state.built_for = Some(selected_type.clone());
    }

    for mut sprite in &mut sprites {
        sprite.color = tint;
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
