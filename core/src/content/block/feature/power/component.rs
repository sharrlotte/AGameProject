use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct Supply {
    pub value: f32,
}

impl Default for Supply {
    fn default() -> Self {
        Self { value: 1.0 }
    }
}

impl Supply {
    pub fn value(&self) -> f32 {
        self.value
    }
}

#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct Boost {
    pub value: f32,
}

impl Default for Boost {
    fn default() -> Self {
        Self { value: 1.0 }
    }
}

impl Boost {
    pub fn value(&self) -> f32 {
        self.value
    }
}

#[derive(Component, Clone, Copy, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct Demand {
    pub value: f32,
}

#[derive(Component, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize, Debug)]
#[reflect(Component)]
pub struct Generator {
    pub output: f32,
}

impl Generator {
    pub fn insert(&self, commands: &mut Commands, base: Entity) {
        commands.entity(base).insert((*self, Boost::default()));
    }
}

#[derive(Component, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize, Debug)]
#[reflect(Component)]
pub struct Battery {
    pub capacity: f32,
}

impl Battery {
    pub fn insert(&self, commands: &mut Commands, base: Entity) {
        commands.entity(base).insert((
            *self,
            Charge {
                value: self.capacity.max(0.0),
            },
        ));
    }
}

#[derive(Component, Clone, Copy, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct Charge {
    pub value: f32,
}

#[derive(Component, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize, Debug)]
#[reflect(Component)]
pub struct Consumer {
    pub rate: f32,
}

impl Consumer {
    pub fn rate(&self) -> f32 {
        self.rate
    }

    pub fn insert(&self, commands: &mut Commands, base: Entity) {
        commands
            .entity(base)
            .insert((*self, Demand::default(), Supply::default()));
    }
}

#[derive(Component, Clone, Copy, PartialEq, Reflect, Serialize, Deserialize, Debug)]
#[reflect(Component)]
pub struct Booster {
    pub radius: f32,
    pub boost: f32,
    pub rate: f32,
}

impl Booster {
    pub fn insert(&self, commands: &mut Commands, base: Entity) {
        commands
            .entity(base)
            .insert((*self, BoosterState::default()));
    }
}

#[derive(Component, Clone, Copy, Debug, Reflect, Default)]
#[reflect(Component)]
pub struct BoosterState {
    pub powered: bool,
}
