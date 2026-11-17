use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::content::block::{
    component::{BlockDef, ShootPattern, TargetingStrategy},
    feature::{ammo::Ammo, efficiency::BlockEfficiency},
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
        ));

        self.shooting_pattern.insert(commands, base);

        if let Some(ammo) = &self.ammo {
            ammo.insert(commands, base);
        }

        let barrel = commands
            .spawn((
                Transform::from_translation(self.offset.extend(self.z)),
                Sprite {
                    image: self.image.clone(),
                    custom_size: Some(def.get_pixel_size()),
                    ..default()
                },
                TurretSprite,
            ))
            .id();
        commands.entity(base).add_child(barrel);
        commands.entity(base).insert(TurretBarrel(barrel));
    }
}

#[derive(Component, Debug, Clone, Copy, Reflect)]
pub struct TurretSprite;

#[derive(Component, Debug, Clone, Copy, Reflect)]
pub struct TurretBarrel(pub Entity);

#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct Turret {
    pub range: f32,
    pub turn_speed: f32,
    pub barrels: Vec<Vec2>,
    pub strategy: TargetingStrategy,
    pub shooting_pattern: ShootPattern,
}

/// What a turret is aiming at. Ship targets are homed; projectile targets are
/// point-defense intercepts and are never persisted.
#[derive(Debug, Clone, Copy, Reflect, PartialEq, Eq, Serialize, Deserialize)]
pub enum TurretTarget {
    Ship(Entity),
    Projectile(Entity),
}

#[derive(Component, Debug, Clone, Reflect, Serialize, Deserialize, Default)]
pub struct TurretState {
    pub active: bool,
    #[serde(skip)]
    pub target: Option<TurretTarget>,
    pub aim_point: Vec2,
    pub can_fire: bool,
    pub shoot_at: Option<Vec2>,
}

impl TurretState {
    pub fn default() -> Self {
        Self {
            active: false,
            target: None,
            aim_point: Vec2::ZERO,
            can_fire: false,
            shoot_at: None,
        }
    }
}
