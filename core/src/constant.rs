use bevy::color::Color;

pub const GRID_SIZE: f32 = 64.0;
pub const SMALL_GRID_SIZE: f32 = 6.0;

pub const FLASH_OVERLAY_COLOR: Color = Color::srgba(5.0, 5.0, 5.0, 0.8);
pub const AUDIO_SCALE: f32 = 1. / 8000.0;

pub const LAYER_BULLET: f32 = 100.0;
pub const LAYER_DROP: f32 = 110.0;
pub const LAYER_FLASH_OVERLAY: f32 = 200.0;
pub const LAYER_BLOCK_ICON: f32 = 210.0;
pub const LAYER_DAMAGE_TEXT: f32 = 400.0;

pub const DROP_PICKUP_RADIUS: f32 = 32.0;
