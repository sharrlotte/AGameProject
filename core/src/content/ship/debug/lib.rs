use bevy::prelude::*;

use super::ShipDebugSelection;

pub fn is_selected(selection: &ShipDebugSelection, entity: Entity) -> bool {
    selection.selected.contains(&entity)
}
