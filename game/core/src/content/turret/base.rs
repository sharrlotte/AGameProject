use bevy::prelude::*;
use std::collections::HashSet;

use crate::{
    content::block::base::{ShootPattern, TargetingStrategy},
    save::io::Save,
};

#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct Turret {
    pub range: f32,
    pub turn_speed: f32,
    pub barrels: Vec<Vec2>,
    pub fire_all_at_once: bool,
    pub strategy: TargetingStrategy,
    pub shooting_pattern: ShootPattern,
}

#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
#[require(Save)]
pub struct TurretState {
    #[reflect(ignore)]
    pub potential_targets: HashSet<Entity>,

    pub active: bool,
    pub target: Option<Entity>,
    pub shoot_at: Option<Vec2>,
}

#[derive(Component, Default, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct PlayerFireController;

#[derive(Component, Default, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct AIFireController;
