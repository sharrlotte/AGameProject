use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::GameConfigValue;
use crate::content::block::{feature::efficiency::BlockEfficiency, stat::StatEffect};

// ThrustPower
#[derive(Component, Clone, Copy, Debug, Reflect, Default)]
pub struct ThrustPowerBase {
    value: f32,
}

impl ThrustPowerBase {
    pub fn value(&self) -> f32 {
        self.value
    }
}

#[derive(Component, Clone, Copy, Debug, Reflect, Default)]
pub struct ThrustPowerComputed {
    value: f32,
}

impl ThrustPowerComputed {
    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn set(&mut self, value: f32) {
        self.value = value;
    }
}

#[derive(Component, Clone, Debug, Reflect, Default)]
pub struct ThrustPowerEffects(pub Vec<StatEffect>);

// RotateSpeed
#[derive(Component, Clone, Copy, Debug, Reflect, Default)]
pub struct RotateSpeedBase {
    value: f32,
}

impl RotateSpeedBase {
    pub fn value(&self) -> f32 {
        self.value
    }
}

#[derive(Component, Clone, Copy, Debug, Reflect, Default)]
pub struct RotateSpeedComputed {
    value: f32,
}

impl RotateSpeedComputed {
    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn set(&mut self, value: f32) {
        self.value = value;
    }
}

#[derive(Component, Clone, Debug, Reflect, Default)]
pub struct RotateSpeedEffects(pub Vec<StatEffect>);

// #[derive(Component, Debug, Clone, Reflect)]
// pub struct Thruster {
//     force: f32,
//     torque: f32,
// }

// impl Thruster {
//     pub fn new(force: f32, torque: f32) -> Self {
//         Self { force, torque }
//     }
// }

#[derive(Component, Debug, Clone, Reflect, PartialEq, Serialize, Deserialize)]
pub struct Thruster {
    force: f32,
    torque: f32,
}

impl Thruster {
    pub fn insert(&self, commands: &mut Commands, base: Entity, config: &GameConfigValue) {
        commands.entity(base).insert((
            ThrustPowerBase {
                value: self.force * config.0.thrust_power_factor,
            },
            ThrustPowerComputed::default(),
            RotateSpeedBase {
                value: self.torque * config.0.turn_speed_factor,
            },
            RotateSpeedComputed::default(),
            BlockEfficiency::default(),
        ));
    }
}
