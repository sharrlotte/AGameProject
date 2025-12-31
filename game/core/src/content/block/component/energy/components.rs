use bevy::prelude::*;
use crate::content::block::stat::StatEffect;

#[derive(Clone, PartialEq, Reflect)]
pub struct EnergyConsumerComponent {
    pub rate: f32,
}

// PowerUsage
#[derive(Component, Clone, Copy, Debug, Reflect, Default)]
pub struct PowerUsageBase {
    pub value: f32,
}

#[derive(Component, Clone, Copy, Debug, Reflect, Default)]
pub struct PowerUsageComputed {
    pub value: f32,
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
