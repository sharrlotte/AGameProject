use bevy::color::Color;

pub const GRID_SIZE: f32 = 64.0;
pub const SMALL_GRID_SIZE: f32 = 6.0;

pub const VIEW_HEIGHT: f32 = 1080.0;

pub const FLASH_OVERLAY_COLOR: Color = Color::srgba(5.0, 5.0, 5.0, 0.8);
pub const AUDIO_SCALE: f32 = 1. / 8000.0;

#[allow(dead_code)]
pub const LAYER_DEBRIS: f32 = 2.0;
#[allow(dead_code)]
pub const LAYER_HULL: f32 = 10.0;
#[allow(dead_code)]
pub const LAYER_TURRET_BASE: f32 = 20.0;
#[allow(dead_code)]
pub const LAYER_TURRET_BARREL: f32 = 50.0;
pub const LAYER_OVERDRIVE_FIELD: f32 = 70.0;
pub const LAYER_BULLET: f32 = 100.0;
#[allow(dead_code)]
pub const LAYER_BEAM: f32 = 110.0;
pub const LAYER_DROP: f32 = 120.0;
pub const LAYER_SHIELD: f32 = 160.0;
#[allow(dead_code)]
pub const LAYER_EXPLOSION: f32 = 200.0;
pub const LAYER_FLASH_OVERLAY: f32 = 220.0;
pub const LAYER_BLOCK_ICON: f32 = 300.0;

pub const DROP_PICKUP_RADIUS: f32 = 32.0;
