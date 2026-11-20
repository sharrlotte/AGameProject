use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::constant::LAYER_TURRET_BARREL;

use crate::content::block::{
    component::{ShootPattern, TargetingStrategy},
    feature::{
        ammo::Ammo,
        power::{Boost, Demand, Supply},
    },
    radar::Radar,
};

#[derive(Clone, Copy)]
pub struct Candidate {
    pub(super) target: TurretTarget,
    pub(super) aim: Vec2,
    pub(super) dist_sq: f32,
    pub(super) health: f32,
    pub(super) priority: f32,
    pub(super) id: u64,
}

#[derive(Component, Debug, Clone, Copy, Reflect, PartialEq, Serialize, Deserialize)]
#[reflect(Component)]
pub enum TurretPower {
    Charge { capacity: f32, charge_rate: f32 },
    Drain { rate: f32 },
}

#[derive(Component, Debug, Clone, Copy, Reflect, Default)]
#[reflect(Component)]
pub struct TurretCharge {
    pub value: f32,
}

#[derive(Debug, Clone, Reflect, PartialEq)]
pub struct TurretComponent {
    pub strategy: TargetingStrategy,
    pub shooting_pattern: ShootPattern,
    pub turn_speed: f32,
    pub aim_tolerance: f32,
    pub range: f32,
    pub barrels: Vec<Vec2>,
    pub image: Handle<Image>,
    pub offset: Vec2,
    pub animated: bool,
    pub ammo: Option<Ammo>,
    pub power: Option<TurretPower>,
}

impl TurretComponent {
    pub fn insert(&self, commands: &mut Commands, base: Entity, visual: Entity, pixel_size: Vec2) {
        commands.entity(base).insert((
            Turret {
                strategy: self.strategy.clone(),
                range: self.range,
                turn_speed: self.turn_speed,
                aim_tolerance: self.aim_tolerance,
                barrels: self.barrels.clone(),
                shooting_pattern: self.shooting_pattern.clone(),
            },
            Boost::default(),
            TurretState::default(),
            Radar::new(self.range, 1.0),
        ));

        self.shooting_pattern.insert(commands, base);

        if let Some(ammo) = &self.ammo {
            ammo.insert(commands, base);
        }

        if let Some(power) = self.power {
            commands
                .entity(base)
                .insert((power, Demand::default(), Supply::default()));
            if let TurretPower::Charge { capacity, .. } = power {
                commands.entity(base).insert(TurretCharge {
                    value: capacity.max(0.0),
                });
            }
        }

        let barrel = commands
            .spawn((
                Transform::from_translation(self.offset.extend(LAYER_TURRET_BARREL)),
                Sprite {
                    image: self.image.clone(),
                    custom_size: Some(pixel_size),
                    ..default()
                },
                TurretSprite,
            ))
            .id();
            
        commands.entity(visual).add_child(barrel);
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
    pub aim_tolerance: f32,
    pub barrels: Vec<Vec2>,
    pub strategy: TargetingStrategy,
    pub shooting_pattern: ShootPattern,
}

#[derive(Debug, Clone, Copy, Reflect, PartialEq, Eq, Serialize, Deserialize)]
pub enum TurretTarget {
    Ship(Entity),
    Projectile(Entity),
}

#[derive(Component, Debug, Clone, Reflect, Serialize, Deserialize, Default)]
#[reflect(Component)]
pub enum TurretState {
    #[default]
    Idle,
    Tracking { aim_point: Vec2, target: Option<TurretTarget> },
    Ready { aim_point: Vec2, target: Option<TurretTarget> },
}

impl TurretState {
    pub fn active(&self) -> bool {
        matches!(self, Self::Ready { .. })
    }

    pub fn aim_point(&self) -> Vec2 {
        match self {
            Self::Idle => Vec2::ZERO,
            Self::Tracking { aim_point, .. } | Self::Ready { aim_point, .. } => *aim_point,
        }
    }

    pub fn target(&self) -> Option<TurretTarget> {
        match self {
            Self::Idle => None,
            Self::Tracking { target, .. } | Self::Ready { target, .. } => *target,
        }
    }

    pub fn track(&mut self, aim_point: Vec2, target: Option<TurretTarget>) {
        *self = Self::Tracking { aim_point, target };
    }

    pub fn ready(&mut self, aim_point: Vec2, target: Option<TurretTarget>) {
        *self = Self::Ready { aim_point, target };
    }

    pub fn clear(&mut self) {
        *self = Self::Idle;
    }
}
