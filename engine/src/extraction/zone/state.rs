use bevy::prelude::*;

#[derive(Component, Debug, Clone, Copy)]
pub struct ExtractionZone {
    pub radius: f32,
    pub hold_duration_secs: f32,
    pub progress_secs: f32,
    pub active: bool,
    pub completed: bool,
}

