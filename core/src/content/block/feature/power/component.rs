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
    pub fn validated(self, block_type: &str) -> Self {
        if self.output < 0.0 {
            warn!(
                "Generator output on {block_type} is negative ({}); clamping to 0.0",
                self.output
            );
            Self { output: 0.0 }
        } else {
            self
        }
    }

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
    pub fn validated(self, block_type: &str) -> Self {
        if self.capacity < 0.0 {
            warn!(
                "Battery capacity on {block_type} is negative ({}); clamping to 0.0",
                self.capacity
            );
            Self { capacity: 0.0 }
        } else {
            self
        }
    }

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
    pub fn validated(self, block_type: &str) -> Self {
        if self.rate < 0.0 {
            warn!(
                "Consumer rate on {block_type} is negative ({}); clamping to 0.0",
                self.rate
            );
            Self { rate: 0.0 }
        } else {
            self
        }
    }

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
    pub fn validated(self, block_type: &str) -> Self {
        let mut out = self;
        if out.radius < 0.0 {
            warn!(
                "Booster radius on {block_type} is negative ({}); clamping to 0.0",
                out.radius
            );
            out.radius = 0.0;
        }
        if out.rate < 0.0 {
            warn!(
                "Booster rate on {block_type} is negative ({}); clamping to 0.0",
                out.rate
            );
            out.rate = 0.0;
        }
        if out.boost < 1.0 {
            warn!(
                "Booster boost on {block_type} is below 1.0 ({}); clamping to 1.0",
                out.boost
            );
            out.boost = 1.0;
        }
        out
    }

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
