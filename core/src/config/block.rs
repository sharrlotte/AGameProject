use std::collections::HashMap;

use bevy::asset::{AssetEvent, AssetServer};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use serde::Deserialize;

use crate::content::block::{
    BlockRegistry, BlockShape,
    component::{
        BlockDef, BurstShootPattern, DropComponent, LaserBeamPattern, PullingBeamPattern,
        ShootPattern, ShootingProjectile, SingleShootPattern, TargetingStrategy,
    },
    feature::{
        ammo::{Ammo, AmmoBuilder},
        component::Feature,
        power::{Battery, Booster, Consumer, Generator},
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
    pub shape: BlockShape,
    pub features: Vec<FeatureConfig>,
    pub drops: Vec<DropComponent>,
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
                let shooting_pattern = config.shooting_pattern.into_pattern(assets);
                let power = config
                    .power
                    .and_then(|p| validate_turret_power(p, &shooting_pattern, block_type));
                Feature::Turret(TurretComponent {
                    strategy: config.strategy,
                    shooting_pattern,
                    turn_speed: config.turn_speed,
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
            FeatureConfig::Thruster(thruster) => Feature::Thruster(thruster),
            FeatureConfig::Storage(config) => Feature::Storage(config.capacity),
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

impl ShootPatternConfig {
    fn into_pattern(self, assets: &AssetServer) -> ShootPattern {
        match self {
            ShootPatternConfig::Single(pattern) => ShootPattern::Single(pattern),
            ShootPatternConfig::Burst(config) => ShootPattern::Burst(BurstShootPattern {
                fire_rate: config.fire_rate,
                delay: config.delay,
                amount: config.amount,
                projectile: config.projectile,
                shoot_sound: config
                    .shoot_sound
                    .map(|sound| assets.load(format!("audio/{sound}"))),
            }),
            ShootPatternConfig::LaserBeam(pattern) => ShootPattern::LaserBeam(pattern),
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
    pub fn into_def(self, assets: &AssetServer, block_type: String) -> BlockDef {
        let features = self
            .features
            .into_iter()
            .map(|feature| feature.into_feature(assets, &block_type))
            .collect();
        BlockDef {
            name: self.name,
            block_type,
            block_group: self.block_group,
            mass: self.mass,
            health: self.health,
            cost: self.cost,
            shape: self.shape,
            features,
            drops: self.drops,
        }
    }
}

pub fn blocks_bridge(
    mut events: MessageReader<AssetEvent<BlocksConfig>>,
    assets: Res<Assets<BlocksConfig>>,
    server: Res<AssetServer>,
    mut registry: ResMut<BlockRegistry>,
) {
    for event in events.read() {
        let id = match event {
            AssetEvent::Modified { id } | AssetEvent::LoadedWithDependencies { id } => *id,
            _ => continue,
        };

        if let Some(config) = assets.get(id) {
            let mut rebuilt = BlockRegistry::default();

            for (key, block) in &config.blocks {
                rebuilt.register(key, block.clone().into_def(server.as_ref(), key.clone()));
            }

            *registry = rebuilt;
        }
    }
}
