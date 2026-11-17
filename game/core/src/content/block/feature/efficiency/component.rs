use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Default, SystemSet, PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub struct EfficiencyUpdateSet;

#[derive(Component, Clone, Copy, Debug, Reflect, Default)]
pub struct EfficiencyEffect {
    value: f32,
}

impl EfficiencyEffect {
    pub fn value(&self) -> f32 {
        self.value
    }
}

#[derive(Component, Clone, Copy, Debug, Reflect)]
pub struct BlockEfficiency {
    value: f32,
    base: f32,
}

impl BlockEfficiency {
    pub fn set(&mut self, value: f32) {
        self.value = value;
        self.base = value;
    }

    pub fn value(&self) -> f32 {
        self.value
    }
}

impl BlockEfficiency {
    pub fn apply(&mut self, value: f32) {
        let buffed = self.base * value;

        if buffed > self.value {
            self.value = buffed;
        }
    }
}

impl Default for BlockEfficiency {
    fn default() -> Self {
        Self {
            value: 1.0,
            base: 1.0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Reflect, Default, PartialEq, Serialize, Deserialize)]
pub struct EfficiencyProvider {
    radius: f32,
    value: f32,
}

impl EfficiencyProvider {
    pub fn radius(&self) -> f32 {
        self.radius
    }

    pub fn value(&self) -> f32 {
        self.value
    }
}


