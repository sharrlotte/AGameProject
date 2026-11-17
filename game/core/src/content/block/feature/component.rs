use bevy::prelude::*;

use crate::config::GameConfigValue;
use crate::content::{
    block::{
        component::BlockDef,
        feature::{
            efficiency::EfficiencyProvider, energy::EnergyConsumer, sprite::Texture,
            storage::Storage, thruster::Thruster, turret::TurretComponent,
        },
    },
    team::Team,
};

#[derive(Component, Clone, PartialEq, Reflect)]
pub enum Feature {
    Texture(Texture),
    Turret(TurretComponent),
    EnergyConsumer(EnergyConsumer),
    Thruster(Thruster),
    EfficiencyProvider(EfficiencyProvider),
    Storage(Storage),
}

impl Feature {
   pub fn insert(
        &self,
        commands: &mut Commands,
        base: Entity,
        def: &BlockDef,
        _team: &Team,
        config: &GameConfigValue,
    ) {
        match self {
            Self::Texture(sprite) => sprite.insert(commands, base, def),
            Self::Turret(turret) => turret.insert(commands, base, def),
            Self::Thruster(thruster) => thruster.insert(commands, base, config),
            Self::EnergyConsumer(consumer) => consumer.insert(commands, base),
            Self::Storage(storage) => storage.insert(commands, base),
            Self::EfficiencyProvider(provider) => {
                commands.entity(base).insert(*provider);
            }
        }
    }

    /// Extra texture layers that cannot share the block's single `Sprite`.
    pub fn extra_texture(&self) -> Option<&Texture> {
        match self {
            Self::Texture(t) => Some(t),
            _ => None,
        }
    }
}
