use std::f32::consts::FRAC_PI_8;

use bevy::prelude::*;
use serde::Deserialize;

use crate::content::block::component::{
    BurstShootPattern, LaserBeamPattern, PullingBeamPattern, ShootPattern, ShootingProjectile,
    SingleShootPattern, TargetingStrategy,
};
use crate::content::block::feature::turret::{TurretComponent, TurretPower};
use crate::validate::{Validate, check};
use super::ammo::AmmoConfig;

#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct TurretComponentConfig {
    pub strategy: TargetingStrategy,
    pub shooting_pattern: ShootPatternConfig,
    pub turn_speed: f32,
    #[serde(default)]
    pub aim_tolerance: Option<f32>,
    pub range: f32,
    pub barrels: Vec<Vec2>,
    pub image: String,
    pub offset: Vec2,
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
    pub capacity: f32,
    pub charge_rate: f32,
}

#[derive(Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct TurretDrainConfig {
    pub rate: f32,
}

#[derive(Deserialize, Clone, PartialEq, Debug)]
pub enum ShootPatternConfig {
    Single(SingleShootPattern),
    Burst(BurstShootPatternConfig),
    LaserBeam(LaserBeamPattern),
    PullingBeam(PullingBeamPattern),
}

#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct BurstShootPatternConfig {
    pub fire_rate: f32,
    pub delay: f32,
    pub amount: i8,
    pub projectile: ShootingProjectile,
    pub shoot_sound: Option<String>,
}

impl Validate for TurretComponentConfig {
    type Context = str;
    type Error = String;

    fn validate(&self, block_type: &str) -> Result<(), String> {
        check::non_negative("turret rotation speed", self.turn_speed)?;
        check::positive("turret targeting range", self.range)?;
        if self.barrels.is_empty() {
            return Err(format!("turret on {block_type} must declare at least one barrel"));
        }
        for (_i, barrel) in self.barrels.iter().enumerate() {
            check::finite("turret barrel horizontal offset", barrel.x)?;
            check::finite("turret barrel vertical offset", barrel.y)?;
        }
        if self.image.trim().is_empty() {
            return Err(format!("turret on {block_type} image path must not be empty"));
        }
        self.shooting_pattern.validate(block_type)?;
        if let Some(ammo) = &self.ammo {
            ammo.validate(&())?;
        }
        Ok(())
    }
}

impl Validate for ShootPatternConfig {
    type Context = str;
    type Error = String;

    fn validate(&self, block_type: &str) -> Result<(), String> {
        match self {
            ShootPatternConfig::Single(p) => {
                check::positive("single shot fire rate", p.fire_rate)?;
                validate_projectile_bounds(&p.projectile, block_type)?;
            }
            ShootPatternConfig::Burst(b) => {
                check::positive("burst fire rate", b.fire_rate)?;
                check::non_negative("burst shot delay", b.delay)?;
                check::at_least("burst projectile count", b.amount, 1)?;
                validate_projectile_bounds(&b.projectile, block_type)?;
            }
            ShootPatternConfig::LaserBeam(lb) => {
                check::non_negative("laser beam damage", lb.damage)?;
                check::positive("laser beam width", lb.width)?;
                check::non_negative("laser beam cooldown", lb.cooldown)?;
                check::positive("laser beam duration", lb.duration)?;
            }
            ShootPatternConfig::PullingBeam(_) => {}
        }
        Ok(())
    }
}

impl TurretComponentConfig {
    pub fn into_turret(self, assets: &AssetServer, block_type: &str) -> TurretComponent {
        let shooting_pattern = self.shooting_pattern.into_pattern(assets, block_type);
        let power = self
            .power
            .and_then(|p| validate_turret_power(p, &shooting_pattern, block_type));

        TurretComponent {
            strategy: self.strategy,
            shooting_pattern,
            turn_speed: self.turn_speed,
            aim_tolerance: self
                .aim_tolerance
                .filter(|t| t.is_finite() && *t > 0.0)
                .unwrap_or(FRAC_PI_8 * 0.5),
            range: self.range,
            barrels: self.barrels,
            image: assets.load(format!("texture/{}", self.image)),
            offset: self.offset,
            animated: self.animated,
            ammo: self.ammo.map(|ammo| ammo.into_ammo(assets)),
            power,
        }
    }
}

impl ShootPatternConfig {
    fn validate_projectile(mut projectile: ShootingProjectile, block_type: &str) -> ShootingProjectile {
        match &mut projectile {
            ShootingProjectile::Bullet { speed, damage, lifetime, piercing, width, length } => {
                if !speed.is_finite() || *speed < 0.0 { warn!("Turret {block_type} projectile speed is invalid; clamping to 0"); *speed = 0.0; }
                if !damage.is_finite() || *damage < 0.0 { warn!("Turret {block_type} projectile damage is invalid; clamping to 0"); *damage = 0.0; }
                if !lifetime.is_finite() || *lifetime < 0.0 { warn!("Turret {block_type} projectile lifetime is invalid; clamping to 0"); *lifetime = 0.0; }
                if *piercing < 0 { warn!("Turret {block_type} piercing is negative; clamping to 0"); *piercing = 0; }
                if !width.is_finite() || *width <= 0.0 { warn!("Turret {block_type} projectile width is invalid; clamping to 1"); *width = 1.0; }
                if !length.is_finite() || *length <= 0.0 { warn!("Turret {block_type} projectile length is invalid; clamping to 1"); *length = 1.0; }
            }
            ShootingProjectile::Missile { speed, damage, lifetime, width, length, .. } => {
                if !speed.is_finite() || *speed < 0.0 { warn!("Turret {block_type} projectile speed is invalid; clamping to 0"); *speed = 0.0; }
                if !damage.is_finite() || *damage < 0.0 { warn!("Turret {block_type} projectile damage is invalid; clamping to 0"); *damage = 0.0; }
                if !lifetime.is_finite() || *lifetime < 0.0 { warn!("Turret {block_type} projectile lifetime is invalid; clamping to 0"); *lifetime = 0.0; }
                if !width.is_finite() || *width <= 0.0 { warn!("Turret {block_type} projectile width is invalid; clamping to 1"); *width = 1.0; }
                if !length.is_finite() || *length <= 0.0 { warn!("Turret {block_type} projectile length is invalid; clamping to 1"); *length = 1.0; }
            }
            ShootingProjectile::Laser { damage, width } => {
                if !damage.is_finite() || *damage < 0.0 { warn!("Turret {block_type} laser damage is invalid; clamping to 0"); *damage = 0.0; }
                if !width.is_finite() || *width <= 0.0 { warn!("Turret {block_type} laser width is invalid; clamping to 1"); *width = 1.0; }
            }
        }
        projectile
    }

    pub fn into_pattern(self, assets: &AssetServer, block_type: &str) -> ShootPattern {
        match self {
            ShootPatternConfig::Single(mut pattern) => {
                pattern.projectile = Self::validate_projectile(pattern.projectile, block_type);
                if !pattern.fire_rate.is_finite() || pattern.fire_rate <= 0.0 {
                    warn!("Turret {block_type} fire_rate is invalid; clamping to 1");
                    pattern.fire_rate = 1.0;
                }
                ShootPattern::Single(pattern)
            }
            ShootPatternConfig::Burst(config) => ShootPattern::Burst(BurstShootPattern {
                fire_rate: if config.fire_rate.is_finite() && config.fire_rate > 0.0 { config.fire_rate } else { 1.0 },
                delay: config.delay.max(0.0),
                amount: config.amount.max(1),
                projectile: Self::validate_projectile(config.projectile, block_type),
                shoot_sound: config
                    .shoot_sound
                    .map(|sound| assets.load(format!("audio/{sound}"))),
            }),
            ShootPatternConfig::LaserBeam(mut pattern) => {
                if !pattern.damage.is_finite() || pattern.damage < 0.0 { pattern.damage = 0.0; }
                if !pattern.width.is_finite() || pattern.width <= 0.0 { pattern.width = 1.0; }
                if !pattern.cooldown.is_finite() || pattern.cooldown < 0.0 { pattern.cooldown = 0.0; }
                if !pattern.duration.is_finite() || pattern.duration <= 0.0 { pattern.duration = 1.0; }
                ShootPattern::LaserBeam(pattern)
            }
            ShootPatternConfig::PullingBeam(pattern) => ShootPattern::PullingBeam(pattern),
        }
    }
}

fn validate_projectile_bounds(projectile: &ShootingProjectile, _block_type: &str) -> Result<(), String> {
    match projectile {
        ShootingProjectile::Bullet { speed, damage, lifetime, piercing, width, length } => {
            check::non_negative("projectile flight speed", *speed)?;
            check::non_negative("projectile impact damage", *damage)?;
            check::non_negative("projectile lifetime in seconds", *lifetime)?;
            check::at_least("projectile piercing count", *piercing, 0)?;
            check::positive("projectile width", *width)?;
            check::positive("projectile length", *length)?;
        }
        ShootingProjectile::Missile { speed, damage, lifetime, width, length, .. } => {
            check::non_negative("missile flight speed", *speed)?;
            check::non_negative("missile impact damage", *damage)?;
            check::non_negative("missile lifetime in seconds", *lifetime)?;
            check::positive("missile width", *width)?;
            check::positive("missile length", *length)?;
        }
        ShootingProjectile::Laser { damage, width } => {
            check::non_negative("laser beam damage", *damage)?;
            check::positive("laser beam width", *width)?;
        }
    }
    Ok(())
}

fn validate_turret_power(
    power: TurretPowerConfig,
    pattern: &ShootPattern,
    block_type: &str,
) -> Option<TurretPower> {
    match (pattern, power) {
        (ShootPattern::Single(_) | ShootPattern::Burst(_), TurretPowerConfig::Charge(c)) => {
            let mut capacity = c.capacity;
            let mut charge_rate = c.charge_rate;
            if capacity < 0.0 {
                warn!("Turret {block_type} power capacity is negative ({capacity}); clamping to 0.0");
                capacity = 0.0;
            }
            if charge_rate < 0.0 {
                warn!("Turret {block_type} power charge_rate is negative ({charge_rate}); clamping to 0.0");
                charge_rate = 0.0;
            }
            Some(TurretPower::Charge {
                capacity,
                charge_rate,
            })
        }
        (
            ShootPattern::LaserBeam(_) | ShootPattern::PullingBeam(_),
            TurretPowerConfig::Drain(drain),
        ) => {
            let mut rate = drain.rate;
            if rate < 0.0 {
                warn!("Turret {block_type} power drain rate is negative ({rate}); clamping to 0.0");
                rate = 0.0;
            }
            Some(TurretPower::Drain { rate })
        }
        _ => {
            warn!(
                "Turret {block_type} power config ({power:?}) is incompatible with pattern ({pattern:?}); ignoring power"
            );
            None
        }
    }
}
