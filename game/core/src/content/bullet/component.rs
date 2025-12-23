use bevy::prelude::*;
use bevy::{
    ecs::{component::Component, entity::Entity},
    time::Timer,
};

use crate::{content::team::Team, save::component::Save};

#[derive(Component, Debug, Clone, Reflect)]
#[require(Save)]
pub struct Bullet {
    pub range: f32,
    pub shooted_at: Vec2,
    pub piercing: i8,
    pub velocity: Vec2,
}

#[derive(Component, Debug, Clone, Reflect)]
#[require(Save)]
pub struct Missile {
    pub lock_range: f32,
    pub shooted_at: Vec2,
    pub speed: f32,
    pub age: f32,
    pub target: Option<Entity>,
    pub accel_time: f32,
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct MissileHitbox {
    pub missile: Entity,
    pub blast_damage: f32,
    pub blast_radius: f32,
    pub team: Team,
}

#[derive(Component, Debug, Clone)]
pub struct Lifetime {
    timer: Timer,
}

impl Lifetime {
    pub fn new(duration: f32) -> Self {
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
