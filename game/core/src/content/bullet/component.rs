use crate::content::team::Team;
use bevy::prelude::*;

/// Team that fired a projectile. Used for point-defense targeting.
#[derive(Component, Debug, Clone, Copy, Reflect)]
pub struct ProjectileOwner {
    pub team: u8,
}

/// Marks projectiles that point defense may intercept. Lasers are excluded.
#[derive(Component, Debug, Clone, Copy, Reflect)]
pub struct Interceptable;

#[derive(Component, Debug, Clone, Reflect)]
pub struct Bullet {
    pub range: f32,
    pub shooted_at: Vec2,
    pub piercing: i8,
    pub velocity: Vec2,
}

#[derive(Component, Debug, Clone, Reflect)]
pub enum Laser {
    Init {
        width: f32,
        length: f32,
        origin: Vec2,
        rotation: Quat,
    },
    Fired {
        fired_at: f32,
    },
}

#[derive(Component, Debug, Clone, Reflect)]
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
