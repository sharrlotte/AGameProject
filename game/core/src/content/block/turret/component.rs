use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::content::block::component::{ShootPattern, TargetingStrategy};

#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct Turret {
    pub range: f32,
    pub turn_speed: f32,
    pub barrels: Vec<Vec2>,
    pub strategy: TargetingStrategy,
    pub shooting_pattern: ShootPattern,
}

#[derive(Component, Debug, Clone, Reflect, Serialize, Deserialize, Default)]
pub struct TurretState {
    pub active: bool,
    pub target: Option<Entity>,
    pub shoot_at: Option<Vec2>,
}

impl TurretState {
    pub fn default() -> Self {
        Self {
            active: false,
            target: None,
            shoot_at: None,
        }
    }
}
