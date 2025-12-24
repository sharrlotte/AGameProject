use bevy::prelude::*;

#[derive(Resource)]
pub struct GameSetting {
    pub volume: f32,
    pub bloom: bool,
}

impl Default for GameSetting {
    fn default() -> Self {
        Self {
            volume: 0.5,
            bloom: true,
        }
    }
}
