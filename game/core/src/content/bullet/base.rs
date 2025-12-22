use bevy::prelude::*;

use crate::{components::death::Death, content::team::Team, save::base::Save};

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

use bevy::{
    app::{App, Plugin, Update},
    ecs::{
        component::Component,
        entity::Entity,
        system::{Commands, Query, Res},
    },
    time::{Time, Timer},
};

pub struct LifetimePlugin;

impl Plugin for LifetimePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_lifetime);
    }
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
}

fn update_lifetime(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Lifetime)>,
) {
    for (entity, mut lifetime) in query.iter_mut() {
        lifetime.timer.tick(time.delta());
        if lifetime.timer.is_finished() {
            let Ok(mut commands) = commands.get_entity(entity) else {
                continue;
            };
            commands.insert(Death);
        }
    }
}
