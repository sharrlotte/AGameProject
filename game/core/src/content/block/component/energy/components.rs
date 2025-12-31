use crate::content::block::stat::StatEffect;
use bevy::prelude::*;

#[derive(Clone, PartialEq, Reflect)]
pub struct EnergyConsumerComponent {
    rate: f32,
}

impl EnergyConsumerComponent {
    pub fn rate(&self) -> f32 {
        self.rate
    }

    pub fn new(rate: f32) -> Self {
        Self { rate }
    }
}

// PowerUsage
#[derive(Component, Clone, Copy, Debug, Reflect, Default)]
pub struct PowerUsageBase {
    value: f32,
}

impl PowerUsageBase {
    pub fn new(value: f32) -> Self {
        Self { value }
    }

    pub fn value(&self) -> f32 {
        self.value
    }
}

#[derive(Component, Clone, Copy, Debug, Reflect, Default)]
pub struct PowerUsageComputed {
    value: f32,
}

impl PowerUsageComputed {
    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn set(&mut self, value: f32) {
        self.value = value;
    }
}

#[derive(Component, Clone, Debug, Reflect, Default)]
pub struct PowerUsageEffects(pub Vec<StatEffect>);

impl EnergyConsumerComponent {
    pub fn insert(&self, commands: &mut Commands, base: Entity) {
        commands.entity(base).insert((
            PowerUsageBase { value: self.rate },
            PowerUsageComputed::default(),
        ));
    }
}
