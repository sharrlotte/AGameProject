use super::*;


use crate::scene::widget::{ButtonVisuals, spawn_button};


#[derive(Component)]
pub struct LoadShipButton(pub String);


pub fn load_initial_ship(
    mut commands: Commands,
    mut editor: ResMut<EditorState>,
    registry: Res<BlockRegistry>,
    catalog: Res<ShipCatalog>,
    placed_blocks: Query<Entity, With<BlockRef>>,
    ship_base: Single<Entity, With<ShipBase>>,
) {
    let Some(design) = catalog.designs().first() else {
        return;
    };

    let filename = ShipCatalog::file_name(design);

    despawn_ship(&mut commands, &placed_blocks);

    spawn_ship(
        &mut commands,
        &mut editor,
        &registry,
        &catalog,
        &filename,
        ship_base.entity(),
    );
}


fn on_load_ship(
    click: On<PointerClick>,
    buttons: Query<&LoadShipButton>,
    mut selections: MessageWriter<LoadShipSelected>,
) {
    let Ok(button) = buttons.get(click.event().entity) else { return; };
    selections.write(LoadShipSelected {
        filename: button.0.clone(),
    });
}


pub fn refresh_ship_list(
    mut commands: Commands,
    mut available_ships: ResMut<EditorState>,
    catalog: Res<ShipCatalog>,
    ship_list_panel: Single<Entity, With<ShipListPanel>>,
) {
    available_ships.ships = catalog.designs().to_vec();

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
                        spawn_button(
                            row,
                            Node {
                                flex_grow: 1.0,
                                height: Val::Percent(100.0),
                                justify_content: JustifyContent::Start,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(1.0)),
                                padding: UiRect::all(Val::Px(8.0)),
                                margin: UiRect::right(Val::Px(4.0)),
                                border_radius: BorderRadius::all(Val::Px(4.0)),
                                ..default()
                            },
                            if ship.name.is_empty() {
                                ShipCatalog::file_name(ship)
                            } else {
                                ship.name.clone()
                            },
                            14.0,
                            Color::WHITE,
                            LoadShipButton(ShipCatalog::file_name(ship)),
                            on_load_ship,
                            ButtonVisuals::dark(),
                        );
                    });
            }
        });
}


pub fn handle_ship_loading(
    mut commands: Commands,
    mut editor: ResMut<EditorState>,
    registry: Res<BlockRegistry>,
    catalog: Res<ShipCatalog>,
    placed_blocks: Query<Entity, With<BlockRef>>,
    mut selections: MessageReader<LoadShipSelected>,
    ship_base: Single<Entity, With<ShipBase>>,
) {
    for selection in selections.read() {
        despawn_ship(&mut commands, &placed_blocks);
        spawn_ship(
            &mut commands,
            &mut editor,
            &registry,
            &catalog,
            &selection.filename,
            ship_base.entity(),
        );
    }
}


pub fn spawn_placed_block(
    commands: &mut Commands,
    base: Entity,
    def: &BlockDef,
    placement: &BlockPlacement,
) -> Entity {
    let transform = def.to_transform(placement.position, placement.rotation);

    let block = commands
        .spawn((
            BlockRef { id: placement.id },
            Visibility::Visible,
            transform,
        ))
        .id();

    commands.entity(base).add_child(block);

    let visual = commands
        .spawn((visual_transform(def.shape.xform()), Visual, Visibility::Visible))
        .id();
        
    commands.entity(block).add_child(visual);

    let mut textures = def.features.iter().filter_map(|f| f.extra_texture());
    if let Some(primary) = textures.next() {
        commands.entity(visual).insert(Sprite {
            image: primary.image.clone(),
            custom_size: Some(def.get_base_pixel_size()),
            ..default()
        });
    }
    for extra in textures {
        let layer = commands
            .spawn((
                Transform::from_translation(extra.offset.extend(extra.z)),
                Sprite {
                    image: extra.image.clone(),
                    custom_size: Some(def.get_base_pixel_size()),
                    ..default()
                },
            ))
            .id();
        commands.entity(visual).add_child(layer);
    }

    for feature in &def.features {
        if let Feature::Turret(turret) = feature {
            let barrel = commands
                .spawn((
                    Transform::from_translation(turret.offset.extend(turret.z)),
                    Sprite {
                        image: turret.image.clone(),
                        custom_size: Some(def.get_base_pixel_size()),
                        ..default()
                    },
                    TurretSprite,
                ))
                .id();
            commands.entity(visual).add_child(barrel);
        }
    }

    block
}


pub fn remove_block_at(
    commands: &mut Commands,
    editor: &mut EditorState,
    registry: &BlockRegistry,
    grid_position: IVec2,
) {
    let Some(grid_block) = editor.grid.block_at(grid_position).cloned() else {
        return;
    };

    let placement = grid_block.placement.clone();

    let Some(def) = registry.get(&placement.block_type) else {
        warn!("No block def {:?}", placement.block_type);
        return;
    };

    editor.grid.remove_block(placement.id);

    commands.entity(grid_block.entity).despawn();
    let _ = def;
}


pub fn despawn_ship(commands: &mut Commands, placed_blocks: &Query<Entity, With<BlockRef>>) {
    for entity in placed_blocks {
        commands.entity(entity).despawn();
    }
}


pub fn spawn_ship(
    commands: &mut Commands,
    editor: &mut EditorState,
    registry: &BlockRegistry,
    catalog: &ShipCatalog,
    filename: &str,
    ship_base: Entity,
) {
    editor.grid = BlockGrid::new();
    editor.filename = filename.to_string();

    if let Some(ship_to_load) = catalog
        .designs()
        .iter()
        .find(|ship| ShipCatalog::file_name(ship) == filename)
        .cloned()
    {
        editor.filename = ship_to_load.name.clone();
        editor.doctrine = ship_to_load.doctrine;

        if ship_to_load.blocks.is_empty() {
            warn!("Ship {} is empty", editor.filename);
            return;
        }

        for saved_block in ship_to_load.blocks {
            if let Some(def) = registry.get(&saved_block.block_type) {
                let placement = saved_block;
                let base = spawn_placed_block(commands, ship_base, def, &placement);
                editor.grid.insert_block(placement, base, def);
            }
        }
    } else {
        error!("Ship {} data not found", filename);
    }
}
