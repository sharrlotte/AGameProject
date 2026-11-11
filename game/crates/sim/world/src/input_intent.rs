use bevy::prelude::*;

/// Intent produced by local input (or future network/AI) and consumed by simulation systems.
#[derive(Component, Default, Debug, Clone)]
pub struct ActionIntent {
    /// Normalized desired movement direction on the XZ plane (`x` = world X, `y` = world Z).
    pub move_dir: Vec2,
    pub sprint: bool,
    /// Cursor intersection point on the horizontal aim plane (`Y = PlayerConfig::aim_height`).
    pub aim_world: Option<Vec3>,
    /// Right mouse button held; requests the combat/aim stance.
    pub aim_held: bool,
    pub fire_held: bool,
    pub fire_just_pressed: bool,
    pub reload_just_pressed: bool,
    /// Optional weapon slot index requested (`0` = Rifle, `1` = Shotgun).
    pub swap_slot: Option<usize>,
}
