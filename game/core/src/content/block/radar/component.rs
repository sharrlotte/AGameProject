use bevy::prelude::*;

#[derive(Component)]
pub struct Radar {
    pub range: f32,
    pub timer: Timer,
    pub targets: Vec<Entity>,
}

impl Radar {
    pub fn new(range: f32, interval: f32) -> Self {
        Self {
            range,
            timer: Timer::from_seconds(interval, TimerMode::Repeating),
            targets: Vec::new(),
        }
    }
}
