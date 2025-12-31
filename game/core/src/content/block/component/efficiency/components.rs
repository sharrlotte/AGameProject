use bevy::prelude::*;

#[derive(Default, SystemSet, PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub struct EfficiencyUpdateSet;

#[derive(Component, Clone, Copy, Debug, Reflect, Default)]
pub struct EfficiencyEffect {
    pub value: f32,
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

/// Component that provides efficiency buff to nearby blocks.
#[derive(Component, Clone, Copy, Debug, Reflect, Default, PartialEq)]
pub struct EfficiencyProvider {
    pub radius: f32,
    pub value: f32,
}
