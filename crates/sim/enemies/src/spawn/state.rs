use bevy::prelude::*;

#[derive(Resource, Debug, Default)]
pub struct EnemySpawner {
    pub timer: f32,
}
