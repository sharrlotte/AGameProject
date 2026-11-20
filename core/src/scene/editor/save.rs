use super::*;

use crate::save::design::DesignSaveRequested;

pub fn handle_ship_saving(
    mut design_saves: MessageWriter<DesignSaveRequested>,
    state: Res<EditorState>,
    registry: Res<BlockRegistry>,
) {
    let layout: Vec<BlockPlacement> = state
        .grid
        .blocks()
        .map(|block| block.placement.clone())
        .collect();

    if layout.len() <= 1 {
        return;
    }

    if let Err(errors) = registry.validate_ship(&layout) {
        warn!(
            "Refusing to save invalid ship {:?}: {:?}",
            state.filename, errors
        );
        return;
    }

    perform_save_schematic(
        Uuid::now_v7(),
        state.filename.clone(),
        &state.grid,
        state.doctrine,
        &mut design_saves,
    );
}

pub fn perform_save_schematic(
    id: Uuid,
    name: String,
    grid: &BlockGrid,
    doctrine: Doctrine,
    design_saves: &mut MessageWriter<DesignSaveRequested>,
) {
    let design = ShipDesign {
        version: DESIGN_SCHEMA_VERSION,
        id,
        name,
        blocks: grid.blocks().map(|b| b.placement.clone()).collect(),
        doctrine,
    };

    design_saves.write(DesignSaveRequested { design });
}
