use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::content::block::base::{ShootPattern, TargetingStrategy};

#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct Turret {
    pub range: f32,
    pub turn_speed: f32,
    pub barrels: Vec<Vec2>,
    pub strategy: TargetingStrategy,
    pub shooting_pattern: ShootPattern,
}

#[derive(Component, Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct TurretState {
    pub active: bool,
    pub target: Option<Entity>,
    pub shoot_at: Option<Vec2>,
}
