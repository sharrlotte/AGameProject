use bevy::prelude::*;

#[derive(Component)]
pub struct ArenaStaticEntity;

#[derive(Component)]
pub struct ArenaGround;

#[derive(Component)]
pub struct ArenaObstacle {
    pub half_extents: Vec3,
    pub is_wall: bool,
}

