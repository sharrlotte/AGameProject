use bevy::prelude::*;

use crate::{
    constant::{THRUST_POWER_FACTOR, TURN_SPEED_FACTOR},
    content::block::{feature::efficiency::components::BlockEfficiency, stat::StatEffect},
};

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

#[derive(Component, Debug, Clone, Reflect, PartialEq)]
pub struct Thruster {
    force: f32,
    torque: f32,
}

impl Thruster {
    pub fn new(force: f32, torque: f32) -> Self {
        Self { force, torque }
    }

    pub fn insert(&self, commands: &mut Commands, base: Entity) {
        commands.entity(base).insert((
            ThrustPowerBase {
                value: self.force * THRUST_POWER_FACTOR,
            },
            ThrustPowerComputed::default(),
            RotateSpeedBase {
                value: self.torque * TURN_SPEED_FACTOR,
            },
            RotateSpeedComputed::default(),
            BlockEfficiency::default(),
        ));
    }
}
