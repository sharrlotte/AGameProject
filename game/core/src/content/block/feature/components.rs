use crate::content::{
    block::{
        feature::{
            efficiency::EfficiencyProvider, energy::EnergyConsumer, sprite::Texture,
            storage::Storage, thruster::Thruster, turret::TurretComponent,
        },
        components::BlockDef,
    },
    team::Team,
};
use bevy::prelude::*;

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
    pub fn spawn(&self, commands: &mut Commands, base: Entity, def: &BlockDef, team: &Team) {
        let component = commands
            .spawn((
                Transform::default(),
                GlobalTransform::default(),
                Visibility::Inherited,
                *team,
            ))
            .id();

        commands.entity(base).add_child(component);

        match self {
            Self::Texture(sprite) => sprite.insert(commands, component, def),
            Self::Turret(turret) => turret.insert(commands, component, def),
            Self::Thruster(thruster) => thruster.insert(commands, component),
            Self::EnergyConsumer(consumer) => consumer.insert(commands, component),
            Self::Storage(storage) => storage.insert(commands, component),
            Self::EfficiencyProvider(provider) => {
                commands.entity(component).insert(*provider);
            }
        }
    }
}
