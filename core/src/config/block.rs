use std::collections::HashMap;


use bevy::asset::{AssetEvent, AssetServer};

use bevy::prelude::*;

use bevy::reflect::TypePath;

use serde::Deserialize;


use crate::config::{ConfigError, ConfigFile, ConfigState};
use crate::content::block::{
    BlockRegistry,
    component::{
        BlockDef, BakedShape, BurstShootPattern, LaserBeamPattern, Loot, PullingBeamPattern,
        ShapeError, ShapeSpec, ShootPattern, ShootingProjectile, SingleShootPattern,
        TargetingStrategy,
    },
    feature::{
        ammo::{Ammo, AmmoBuilder},
        component::Feature,
        power::{Battery, Booster, Consumer, Generator},
        shield::{Shield, ShieldShape},
        sprite::Texture,
        thruster::Thruster,
        turret::{TurretComponent, TurretPower},
    },
};


#[derive(Asset, TypePath, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct BlocksConfig {
    pub blocks: HashMap<String, BlockDefConfig>,
}


#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct BlockDefConfig {
    pub name: String,
    pub block_group: String,
    pub mass: f32,
    pub health: f32,
    pub cost: u32,
    pub shape: ShapeSpec,
    pub features: Vec<FeatureConfig>,
    pub drops: Vec<Loot>,
}


#[derive(Deserialize, Clone, PartialEq, Debug)]
pub enum FeatureConfig {
    Texture(TextureConfig),
    Turret(TurretComponentConfig),
    Generator(Generator),
    Battery(Battery),
    Consumer(Consumer),
    Booster(Booster),
    Thruster(Thruster),
    Storage(StorageConfig),
    Shield(ShieldConfig),
}


#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct TextureConfig {
    pub image: String,
    pub offset: Vec2,
    pub z: f32,
    pub animated: bool,
}


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
    pub z: f32,
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


#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct AmmoConfig {
    pub capacity: u32,
    pub crafting: HashMap<String, u32>,
    pub icon: Option<String>,
}


#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct StorageConfig {
    pub capacity: u32,
}


#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct ShieldConfig {
    pub shape: ShieldShapeConfig,
    pub max_hp: f32,
    pub cooldown: f32,
    pub regen_rate: f32,
    #[serde(default)]
    pub demand: Option<f32>,
}


#[derive(Deserialize, Clone, PartialEq, Debug)]
pub enum ShieldShapeConfig {
    Circle {
        radius: f32,
    },
    Plane {
        width: f32,
        #[serde(default = "default_plane_thickness")]
        thickness: f32,
        #[serde(default)]
        offset: Vec2,
    },
}





impl FeatureConfig {
    fn into_feature(self, assets: &AssetServer, block_type: &str) -> Feature {
        match self {
            FeatureConfig::Texture(config) => Feature::Texture(Texture {
                image: assets.load(format!("texture/{}", config.image)),
                offset: config.offset,
                z: config.z,
                animated: config.animated,
            }),
            FeatureConfig::Turret(config) => {
                let shooting_pattern = config.shooting_pattern.into_pattern(assets, block_type);

                let power = config
                    .power
                    .and_then(|p| validate_turret_power(p, &shooting_pattern, block_type));
                    
                Feature::Turret(TurretComponent {
                    strategy: config.strategy,
                    shooting_pattern,
                    turn_speed: config.turn_speed,
                    aim_tolerance: config
                        .aim_tolerance
                        .filter(|t| t.is_finite() && *t > 0.0)
                        .unwrap_or(std::f32::consts::FRAC_PI_8 * 0.5),
                    range: config.range,
                    barrels: config.barrels,
                    image: assets.load(format!("texture/{}", config.image)),
                    offset: config.offset,
                    z: config.z,
                    animated: config.animated,
                    ammo: config.ammo.map(|ammo| ammo.into_ammo(assets)),
                    power,
                })
            }
            FeatureConfig::Generator(generator) => {
                Feature::Generator(generator.validated(block_type))
            }
            FeatureConfig::Battery(battery) => Feature::Battery(battery.validated(block_type)),
            FeatureConfig::Consumer(consumer) => Feature::Consumer(consumer.validated(block_type)),
            FeatureConfig::Booster(booster) => Feature::Booster(booster.validated(block_type)),
            FeatureConfig::Thruster(thruster) => Feature::Thruster(thruster.validated()),
            FeatureConfig::Storage(config) => Feature::Storage(config.capacity),
            FeatureConfig::Shield(config) => {
                let shape = match config.shape {
                    ShieldShapeConfig::Circle { radius } => ShieldShape::Circle { radius },
                    ShieldShapeConfig::Plane {
                        width,
                        thickness,
                        offset,
                    } => ShieldShape::Plane {
                        width,
                        thickness,
                        offset,
                    },
                };
                Feature::Shield(Shield::new(
                    shape,
                    config.max_hp,
                    config.cooldown,
                    config.regen_rate,
                    config.demand,
                ))
            }
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
    fn into_pattern(self, assets: &AssetServer, block_type: &str) -> ShootPattern {
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


impl AmmoConfig {
    fn into_ammo(self, assets: &AssetServer) -> Ammo {
        AmmoBuilder::build(AmmoBuilder {
            capacity: self.capacity,
            crafting: self.crafting,
            icon: self.icon.map(|icon| assets.load(format!("texture/{icon}"))),
        })
    }
}


impl BlockDefConfig {
    pub fn into_def(self, assets: &AssetServer, block_type: String) -> Result<BlockDef, ShapeError> {
        let shape = BakedShape::try_from(&self.shape)
            .map_err(|error| ShapeError::new(format!("block {block_type}: {error}")))?;

        let features = self
            .features
            .into_iter()
            .map(|feature| feature.into_feature(assets, &block_type))
            .collect();
            
        Ok(BlockDef {
            name: self.name,
            block_type,
            block_group: self.block_group,
            mass: self.mass,
            health: self.health,
            cost: self.cost,
            shape,
            features,
            drops: self.drops,
        })
    }
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
            TurretPowerConfig::Drain(d),
        ) => {
            let mut rate = d.rate;
            if rate < 0.0 {
                warn!("Turret {block_type} power drain rate is negative ({rate}); clamping to 0.0");
                rate = 0.0;
            }
            Some(TurretPower::Drain { rate })
        }
        _ => {
            warn!(
                "Turret {block_type} has mismatched power config for its shooting pattern; spawning unpowered"
            );
            None
        }
    }
}


pub fn blocks_bridge(
    mut events: MessageReader<AssetEvent<BlocksConfig>>,
    assets: Res<Assets<BlocksConfig>>,
    server: Res<AssetServer>,
    mut registry: ResMut<BlockRegistry>,
    mut config_state: ResMut<NextState<ConfigState>>,
) {
    for event in events.read() {
        let id = match event {
            AssetEvent::Modified { id } | AssetEvent::LoadedWithDependencies { id } => *id,
            _ => continue,
        };

        if let Some(config) = assets.get(id) {
            let mut rebuilt = BlockRegistry::default();
            let mut failed = None;

            for (key, block) in &config.blocks {
                match block.clone().into_def(server.as_ref(), key.clone()) {
                    Ok(def) => rebuilt.register(key, def),
                    Err(error) => {
                        failed = Some(error);
                        break;
                    }
                }
            }

            if let Some(error) = failed {
                config_state.set(ConfigState::Error(ConfigError::with_detail(
                    ConfigFile::Blocks,
                    error.message(),
                )));
                return;
            }

            *registry = rebuilt;
        }
    }
}

fn default_plane_thickness() -> f32 {
    8.0
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shipped_block_config_parses_and_bakes() {
        let config: BlocksConfig = ron::from_str(include_str!(
            "../../assets/config/base/block.ron"
        ))
        .expect("shipped block config parses");
        for (key, block) in &config.blocks {
            BakedShape::try_from(&block.shape)
                .unwrap_or_else(|error| panic!("block {key} failed to bake: {error}"));
        }
    }
}
