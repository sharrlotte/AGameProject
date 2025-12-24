use bevy::prelude::*;

#[derive(Resource)]
pub struct GameSetting {
    pub volumne: f32,
}

impl Default for GameSetting {
    fn default() -> Self {
        Self { volumne: 0.5 }
    }
}
