use bevy::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct Lifetime {
    timer: Timer,
}

impl Lifetime {
    pub fn seconds(duration: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
    pub fn timer(&self) -> &Timer {
        &self.timer
    }

    pub fn tick(&mut self, time: &Time) {
        self.timer.tick(time.delta());
    }
}
