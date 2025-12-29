use bevy::prelude::*;

use crate::common::death::Death;

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

pub struct LifetimePlugin;

impl Plugin for LifetimePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, update_lifetime);
    }
}

pub fn update_lifetime(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Lifetime)>,
) {
    for (entity, mut lifetime) in query.iter_mut() {
        lifetime.tick(&time);

        if lifetime.timer().is_finished() {
            let Ok(mut commands) = commands.get_entity(entity) else {
                continue;
            };

            commands.insert(Death);
        }
    }
}
