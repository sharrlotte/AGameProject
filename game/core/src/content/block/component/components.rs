use crate::content::{
    block::{
        component::{
            efficiency::EfficiencyProvider, energy::EnergyConsumerComponent,
            sprite::SpriteComponent, storage::StorageComponent, thruster::ThrusterComponent,
            turret::TurretComponent,
        },
        components::BlockDef,
    },
    team::Team,
};
use bevy::prelude::*;

#[derive(Component, Clone, PartialEq, Reflect)]
pub enum BlockComponent {
    Sprite(SpriteComponent),
    Turret(TurretComponent),
    EnergyConsumer(EnergyConsumerComponent),
    Thruster(ThrusterComponent),
    EfficiencyProvider(EfficiencyProvider),
    Storage(StorageComponent),
}

impl BlockComponent {
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
            Self::Sprite(sprite) => sprite.spawn(commands, component, def),
            Self::Turret(turret) => turret.spawn(commands, component, def),
            Self::Thruster(thruster) => thruster.spawn(commands, component),
            Self::EnergyConsumer(consumer) => consumer.spawn(commands, component),
            Self::EfficiencyProvider(provider) => {
                commands.entity(component).insert(*provider);
            }
            Self::Storage(storage) => storage.spawn(commands, component),
        }
    }
}
