use crate::content::block::stat::StatEffect;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// A block that produces power from nothing. Shared by config and runtime so
/// there is a single concept of a producer.
#[derive(Component, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize, Debug)]
pub struct PowerProducer {
    pub output: f32,
}

impl PowerProducer {
    pub fn insert(&self, commands: &mut Commands, base: Entity) {
        commands.entity(base).insert(*self);
    }
}

/// A block that consumes power. Shared by config and runtime.
#[derive(Component, Clone, PartialEq, Reflect, Serialize, Deserialize, Debug)]
pub struct EnergyConsumer {
    pub rate: f32,
}

/// Per-block efficiency assigned by the equal-share power solver.
#[derive(Component, Clone, Copy, Debug, Reflect, Default)]
pub struct PowerEfficiency {
    pub value: f32,
}

impl PowerEfficiency {
    pub fn value(&self) -> f32 {
        self.value
    }
}

// PowerUsage
#[derive(Component, Clone, Copy, Debug, Reflect, Default)]
pub struct PowerUsageBase {
    value: f32,
}

impl PowerUsageBase {
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

impl EnergyConsumer {
    pub fn rate(&self) -> f32 {
        self.rate
    }

    pub fn insert(&self, commands: &mut Commands, base: Entity) {
        commands.entity(base).insert((
            PowerUsageBase { value: self.rate },
            PowerUsageComputed::default(),
            PowerEfficiency { value: 1.0 },
        ));
    }
}
