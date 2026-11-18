pub mod ai_panel;
pub mod grid;
pub mod input;
pub mod save;
pub mod ship_list;
pub mod ui;

pub use ai_panel::*;
pub use grid::*;
pub use input::*;
pub use save::*;
pub use ship_list::*;
pub use ui::*;

use bevy::{
    input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
    picking::hover::HoverMap,
    prelude::*,
    window::PrimaryWindow,
};
use ron::ser::{PrettyConfig, to_string_pretty};
use std::mem;
use uuid::Uuid;

use crate::{
    constant::GRID_SIZE,
    content::{
        block::{
            BlockPlacement, BlockRef,
            component::{BlockDef, BlockRegistry, FACE_DOWN, FACE_LEFT, FACE_RIGHT, FACE_UP},
            feature::{Feature, turret::TurretSprite},
        },
        ship::ai::{AiProfile, PRESETS},
        ship::{ShipDesign, ShipGrid},
    },
    resource::{DeployPayload, SceneState},
    save::design::{DESIGN_SCHEMA_VERSION, ShipCatalog},
    scene::layout::{
        BUTTON_WIDTH, CORNER_RADIUS, FONT_SMALL, FONT_TITLE, ICON_SIZE, PANEL_WIDTH, ROW_HEIGHT,
    },
    simulation::SimulationSet,
};

#[derive(Component)]
pub struct EditorUiRoot;

#[derive(Component)]
pub struct PlacementPreview;

#[derive(Component)]
pub struct BlockListPanel;

#[derive(Component)]
pub struct ShipListPanel;

#[derive(Component)]
pub struct ShipBase;

#[derive(Eq, Default, Clone, Copy, PartialEq, Debug)]
pub enum EditorTab {
    #[default]
    Blocks,
    Ships,
    Ai,
}

#[derive(Component)]
pub struct AiPanel;

#[derive(Component)]
pub struct AiPreviewText;

#[derive(Component)]
pub struct BlockPropertiesText;

#[derive(Message)]
pub struct ShipSavesChanged;

#[derive(Message)]
pub struct ShipSaveChanged;

#[derive(Message)]
pub struct LoadShipSelected {
    pub filename: String,
}

pub struct ShipEditorPlugin;

impl Plugin for ShipEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ShipSavesChanged>()
            .add_message::<ShipSaveChanged>()
            .add_message::<LoadShipSelected>()
            .init_resource::<EditorState>()
            .add_systems(
                OnEnter(SceneState::ShipEditor),
                (setup_editor_ui, load_initial_ship, refresh_ship_list).chain(),
            )
            .add_systems(
                Update,
                (
                    sync_block_selection,
                    sync_tabs,
                    handle_ship_loading,
                    process_input,
                    draw_grid,
                    update_placement_preview,
                    handle_placement,
                    handle_block_deletion,
                    update_block_properties_panel,
                    sync_deploy_button,
                    sync_ai_preset,
                    update_ai_preview,
                    handle_ship_saving.run_if(on_message::<ShipSaveChanged>),
                    refresh_ship_list.run_if(on_message::<ShipSavesChanged>),
                )
                    .chain()
                    .run_if(in_state(SceneState::ShipEditor))
                    .in_set(SimulationSet::Input),
            )
            .add_systems(Update, send_scroll_events.in_set(SimulationSet::Input))
            .add_observer(on_scroll_handler);
    }
}

#[derive(Resource, Default)]
pub struct EditorState {
    pub active_tab: EditorTab,
    pub selected_block: Option<String>,
    pub hovered_grid_position: Option<IVec2>,
    pub place_req: bool,
    pub delete_req: bool,
    pub is_dragging: bool,
    pub drag_accumulator: f32,
    pub filename: String,
    pub grid: ShipGrid,
    pub ships: Vec<ShipDesign>,
    pub ai_profile: AiProfile,
}

#[derive(EntityEvent, Debug)]
#[entity_event(propagate, auto_propagate)]
pub struct Scroll {
    entity: Entity,
    delta: Vec2,
}
