use bevy::{
    app::{App, Plugin, Update},
    ecs::{
        component::Component,
        entity::Entity,
        system::{Commands, Query, Res},
    },
    time::{Time, Timer},
};

use crate::components::death::Death;

pub struct LifetimePlugin;

impl Plugin for LifetimePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_lifetime);
    }
}

#[derive(Component, Debug, Clone)]
pub struct Lifetime {
    pub timer: Timer,
}

fn update_lifetime(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Lifetime)>,
) {
    for (entity, mut lifetime) in query.iter_mut() {
        lifetime.timer.tick(time.delta());
        if lifetime.timer.is_finished() {
            commands.entity(entity).insert(Death);
        }
    }
}
