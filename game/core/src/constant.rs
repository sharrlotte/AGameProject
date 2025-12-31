use bevy::color::Color;

pub const GRID_SIZE: f32 = 64.0;
pub const SMALL_GRID_SIZE: f32 = 6.0;
pub const DROP_SIZE: f32 = 10.0;

pub const FLASH_OVERLAY_COLOR: Color = Color::srgba(5.0, 5.0, 5.0, 0.8);
pub const THRUST_POWER_FACTOR: f32 = 50.0;
pub const TURN_SPEED_FACTOR: f32 = 9000.0;
pub const AUDIO_SCALE: f32 = 1. / 8000.0;

pub const LAYER_BLOCK: f32 = 10.0;
pub const LAYER_BLOCK_OVERLAY: f32 = 100.0;
pub const LAYER_BULLET: f32 = 100.0;
pub const LAYER_DROP: f32 = 110.0;
pub const LAYER_TURRET_OVERLAY: f32 = 150.0;
pub const LAYER_FLASH_OVERLAY: f32 = 200.0;
pub const LAYER_BLOCK_ICON: f32 = 210.0;
pub const LAYER_SHIP_UI: f32 = 300.0;

pub const MIN_ZOOM: f32 = 0.2;
pub const MAX_ZOOM: f32 = 30.0;

pub const MAX_SHIPS: usize = 16;
pub const DROP_DESPAWN_IN_SECONDS: f32 = 5.0 * 60.0;
pub const DROP_COLLECT_RADIUS: f32 = 1000.0;

pub const LINEAR_DAMPING: f32 = 0.9;
pub const ANGULAR_DAMPING: f32 = 0.999;
