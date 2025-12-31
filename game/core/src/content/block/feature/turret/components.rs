use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::content::block::{
    feature::{ammo::Ammo, efficiency::BlockEfficiency},
    components::{BlockDef, ShootPattern, TargetingStrategy},
    radar::Radar,
};

#[derive(Debug, Clone, Reflect, PartialEq)]
pub struct TurretComponent {
    pub strategy: TargetingStrategy,
    pub shooting_pattern: ShootPattern,
    pub turn_speed: f32,
    pub range: f32,
    pub barrels: Vec<Vec2>,
    pub image: Handle<Image>,
    pub offset: Vec2,
    pub z: f32,
    pub animated: bool,
    pub ammo: Option<Ammo>,
}

impl TurretComponent {
    pub fn insert(&self, commands: &mut Commands, base: Entity, def: &BlockDef) {
        commands.entity(base).insert((
            Turret {
                strategy: self.strategy.clone(),
                range: self.range,
                turn_speed: self.turn_speed,
                barrels: self.barrels.clone(),
                shooting_pattern: self.shooting_pattern.clone(),
            },
            BlockEfficiency::default(),
            TurretState::default(),
            Radar::new(self.range, 1.0),
            Sprite {
                image: self.image.clone(),
                custom_size: Some(def.get_pixel_size()),
                ..default()
            },
            Transform::from_translation(self.offset.extend(self.z)),
        ));

        self.shooting_pattern.insert(commands, base);

        if let Some(ammo) = &self.ammo {
            ammo.insert(commands, base);
        }
    }
}

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
