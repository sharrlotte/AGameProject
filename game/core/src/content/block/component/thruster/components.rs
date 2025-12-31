use bevy::prelude::*;

use crate::{
    constant::{THRUST_POWER_FACTOR, TURN_SPEED_FACTOR},
    content::block::{
        component::efficiency::BlockEfficiency,
        stat::StatEffect,
    },
};

// ThrustPower
#[derive(Component, Clone, Copy, Debug, Reflect, Default)]
pub struct ThrustPowerBase {
    pub value: f32,
}

#[derive(Component, Clone, Copy, Debug, Reflect, Default)]
pub struct ThrustPowerComputed {
    pub value: f32,
}

#[derive(Component, Clone, Debug, Reflect, Default)]
pub struct ThrustPowerEffects(pub Vec<StatEffect>);

#[derive(Component, Debug, Clone, Reflect)]
pub struct Thruster {
    pub force: f32,
    pub torque: f32,
}

#[derive(Component, Debug, Clone, Reflect, PartialEq)]
pub struct ThrusterComponent {
    pub force: f32,
    pub torque: f32,
}

// RotateSpeed
#[derive(Component, Clone, Copy, Debug, Reflect, Default)]
pub struct RotateSpeedBase {
    pub value: f32,
}

#[derive(Component, Clone, Copy, Debug, Reflect, Default)]
pub struct RotateSpeedComputed {
    pub value: f32,
}

#[derive(Component, Clone, Debug, Reflect, Default)]
pub struct RotateSpeedEffects(pub Vec<StatEffect>);

impl ThrusterComponent {
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
