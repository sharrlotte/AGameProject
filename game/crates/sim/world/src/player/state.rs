use bevy::prelude::*;

#[derive(Component)]
pub struct Survivor;

#[derive(Component)]
pub struct LocalPlayer;

#[derive(Component, Debug, Clone, Copy)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Stamina {
    pub current: f32,
    pub max: f32,
    pub exhausted_timer: f32,
    pub is_sprinting: bool,
}

impl Stamina {
    pub fn new(max: f32) -> Self {
        Self {
            current: max,
            max,
            exhausted_timer: 0.0,
            is_sprinting: false,
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct MovementStats {
    pub walk_speed: f32,
    pub sprint_speed: f32,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct AimDirection(pub Vec3);
