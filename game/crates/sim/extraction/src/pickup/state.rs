use bevy::prelude::*;

#[derive(Component, Debug, Clone, Copy)]
pub struct ScrapPickup {
    pub amount: u32,
    pub radius: f32,
    pub lifetime: f32,
}
