use std::f32::consts::FRAC_PI_8;

use bevy::prelude::*;
use serde::Deserialize;

use crate::content::block::component::{
    BurstShootPattern, LaserBeamPattern, PullingBeamPattern, ShootPattern, ShootingProjectile,
    SingleShootPattern, TargetingStrategy,
};
use crate::content::block::feature::turret::{TurretComponent, TurretPower};
use crate::validate::{FiniteVec2, NonEmpty, NonNegativeF32, PositiveF32, PositiveI8};
use super::ammo::AmmoConfig;

#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct TurretComponentConfig {
    pub strategy: TargetingStrategy,
    pub shooting_pattern: ShootPatternConfig,
    pub turn_speed: NonNegativeF32,
    #[serde(default)]
    pub aim_tolerance: Option<PositiveF32>,
    pub range: PositiveF32,
    pub barrels: NonEmpty<Vec<FiniteVec2>>,
    pub image: NonEmpty<String>,
    pub offset: FiniteVec2,
    pub animated: bool,
    pub ammo: Option<AmmoConfig>,
    pub power: Option<TurretPowerConfig>,
}

#[derive(Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum TurretPowerConfig {
    Charge(TurretChargeConfig),
    Drain(TurretDrainConfig),
}

#[derive(Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct TurretChargeConfig {
    pub capacity: NonNegativeF32,
    pub charge_rate: NonNegativeF32,
}

#[derive(Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct TurretDrainConfig {
    pub rate: NonNegativeF32,
}

#[derive(Deserialize, Clone, PartialEq, Debug)]
pub enum ShootPatternConfig {
    Single(SingleShootPatternConfig),
    Burst(BurstShootPatternConfig),
    LaserBeam(LaserBeamPatternConfig),
    PullingBeam(PullingBeamPatternConfig),
}

#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct SingleShootPatternConfig {
    pub fire_rate: PositiveF32,
    pub projectile: ShootingProjectileConfig,
}

#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct BurstShootPatternConfig {
    pub fire_rate: PositiveF32,
    pub delay: NonNegativeF32,
    pub amount: PositiveI8,
    pub projectile: ShootingProjectileConfig,
    pub shoot_sound: Option<String>,
}

#[derive(Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct LaserBeamPatternConfig {
    pub cooldown: NonNegativeF32,
    pub duration: PositiveF32,
    pub damage: NonNegativeF32,
    pub width: PositiveF32,
}

#[derive(Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct PullingBeamPatternConfig {
    pub width: PositiveF32,
}

#[derive(Deserialize, Clone, PartialEq, Debug)]
pub enum ShootingProjectileConfig {
    Bullet {
        speed: NonNegativeF32,
        damage: NonNegativeF32,
        lifetime: NonNegativeF32,
        piercing: u8,
        width: PositiveF32,
        length: PositiveF32,
    },
    Laser {
        damage: NonNegativeF32,
        width: PositiveF32,
    },
    Missile {
        speed: NonNegativeF32,
        damage: NonNegativeF32,
        lifetime: NonNegativeF32,
        lock_range: f32,
        blast_damage: f32,
        blast_radius: f32,
        width: PositiveF32,
        length: PositiveF32,
    },
}

impl ShootingProjectileConfig {
    fn into_projectile(self) -> ShootingProjectile {
        match self {
            Self::Bullet {
                speed,
                damage,
                lifetime,
                piercing,
                width,
                length,
            } => ShootingProjectile::Bullet {
                speed: speed.get(),
                damage: damage.get(),
                lifetime: lifetime.get(),
                piercing: piercing.min(i8::MAX as u8) as i8,
                width: width.get(),
                length: length.get(),
            },
            Self::Laser { damage, width } => ShootingProjectile::Laser {
                damage: damage.get(),
                width: width.get(),
            },
            Self::Missile {
                speed,
                damage,
                lifetime,
                lock_range,
                blast_damage,
                blast_radius,
                width,
                length,
            } => ShootingProjectile::Missile {
                speed: speed.get(),
                damage: damage.get(),
                lifetime: lifetime.get(),
                lock_range,
                blast_damage,
                blast_radius,
                width: width.get(),
                length: length.get(),
            },
        }
    }
}

impl TurretComponentConfig {
    pub fn into_turret(self, assets: &AssetServer, block_type: &str) -> TurretComponent {
        let shooting_pattern = self.shooting_pattern.into_pattern(assets);
        let power = self
            .power
            .and_then(|p| validate_turret_power(p, &shooting_pattern, block_type));

        TurretComponent {
            strategy: self.strategy,
            shooting_pattern,
            turn_speed: self.turn_speed.get(),
            aim_tolerance: self
                .aim_tolerance
                .map_or(FRAC_PI_8 * 0.5, PositiveF32::get),
            range: self.range.get(),
            barrels: self
                .barrels
                .into_inner()
                .into_iter()
                .map(FiniteVec2::get)
                .collect(),
            image: assets.load(format!("texture/{}", self.image.get())),
            offset: self.offset.get(),
            animated: self.animated,
            ammo: self.ammo.map(|ammo| ammo.into_ammo(assets)),
            power,
        }
    }
}

impl ShootPatternConfig {
    pub fn into_pattern(self, assets: &AssetServer) -> ShootPattern {
        match self {
            ShootPatternConfig::Single(pattern) => ShootPattern::Single(SingleShootPattern {
                fire_rate: pattern.fire_rate.get(),
                projectile: pattern.projectile.into_projectile(),
            }),
            ShootPatternConfig::Burst(config) => ShootPattern::Burst(BurstShootPattern {
                fire_rate: config.fire_rate.get(),
                delay: config.delay.get(),
                amount: config.amount.get(),
                projectile: config.projectile.into_projectile(),
                shoot_sound: config
                    .shoot_sound
                    .map(|sound| assets.load(format!("audio/{sound}"))),
            }),
            ShootPatternConfig::LaserBeam(pattern) => ShootPattern::LaserBeam(LaserBeamPattern {
                cooldown: pattern.cooldown.get(),
                duration: pattern.duration.get(),
                damage: pattern.damage.get(),
                width: pattern.width.get(),
            }),
            ShootPatternConfig::PullingBeam(pattern) => {
                ShootPattern::PullingBeam(PullingBeamPattern {
                    width: pattern.width.get(),
                })
            }
        }
    }
}

fn validate_turret_power(
    power: TurretPowerConfig,
    pattern: &ShootPattern,
    block_type: &str,
) -> Option<TurretPower> {
    match (pattern, power) {
        (ShootPattern::Single(_) | ShootPattern::Burst(_), TurretPowerConfig::Charge(c)) => {
            Some(TurretPower::Charge {
                capacity: c.capacity.get(),
                charge_rate: c.charge_rate.get(),
            })
        }
        (
            ShootPattern::LaserBeam(_) | ShootPattern::PullingBeam(_),
            TurretPowerConfig::Drain(drain),
        ) => Some(TurretPower::Drain {
            rate: drain.rate.get(),
        }),
        _ => {
            warn!(
                "Turret {block_type} power config ({power:?}) is incompatible with pattern ({pattern:?}); ignoring power"
            );
            None
        }
    }
}
