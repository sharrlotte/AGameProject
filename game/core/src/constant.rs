use bevy::color::Color;

pub const GRID_SIZE: f32 = 64.0;
pub const FLASH_OVERLAY_COLOR: Color = Color::srgba(5.0, 5.0, 5.0, 0.8);
pub const THRUST_POWER_FACTOR: f32 = 240.0;
pub const TURN_SPEED_FACTOR: f32 = 27000.0;
pub const AUDIO_SCALE: f32 = 1. / 100.0;

pub const LAYER_BLOCK: f32 = 10.0;
pub const LAYER_BLOCK_OVERLAY: f32 = 100.0;
pub const LAYER_BULLET: f32 = 100.0;
pub const LAYER_TURRET_OVERLAY: f32 = 150.0;
pub const LAYER_FLASH_OVERLAY: f32 = 200.0;
pub const LAYER_SHIP_UI: f32 = 300.0;


pub const MIN_ZOOM: f32 = 0.2;
pub const MAX_ZOOM: f32 = 30.0;

pub const MAX_SHIPS: usize = 10;
