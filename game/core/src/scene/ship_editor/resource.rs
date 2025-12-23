use bevy::{ecs::resource::Resource, math::IVec2};

use crate::{content::{block::component::BlockType, ship::component::ShipGrid}, save::component::SchematicData, scene::ship_editor::component::EditorTab};


#[derive(Resource, Default)]
pub struct EditorState {
    pub active_tab: EditorTab,
    pub selected_block: Option<BlockType>,
    pub hovered_grid_position: Option<IVec2>,
    pub place_req: bool,
    pub delete_req: bool,
    pub is_dragging: bool,
    pub drag_accumulator: f32,
    pub filename: String,
    pub grid: ShipGrid,
    pub ships: Vec<SchematicData>,
}
