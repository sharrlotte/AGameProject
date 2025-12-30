use bevy::prelude::*;

use crate::content::{
    block::{
        base::BlockDef,
        component::{
            efficiency::{EfficiencyPlugin, EfficiencyProvider},
            energy::{EnergyConsumerComponent, EnergyPlugin},
            sprite::SpriteComponent,
            thruster::{ThrusterComponent, ThrusterPlugin},
            turret::TurretComponent,
        },
        turret::TurretPlugin,
    },
    team::Team,
};
pub struct ComponentPlugin;

impl Plugin for ComponentPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ThrusterPlugin, TurretPlugin, EnergyPlugin, EfficiencyPlugin));
    }
}

#[derive(Component, Clone, PartialEq, Reflect)]
pub enum BlockComponent {
    Sprite(SpriteComponent),
    Turret(TurretComponent),
    EnergyConsumer(EnergyConsumerComponent),
    Thruster(ThrusterComponent),
    EfficiencyProvider(EfficiencyProvider),
}

impl BlockComponent {
    pub fn spawn(
        &self,
        commands: &mut Commands,
        ship: Entity,
        base: Entity,
        def: &BlockDef,
        team: &Team,
    ) {
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
            Self::Sprite(sprite) => sprite.spawn(commands, ship, component, def),
            Self::Turret(turret) => turret.spawn(commands, ship, component, def, team),
            Self::Thruster(thruster) => thruster.spawn(commands, ship, component),
            Self::EnergyConsumer(consumer) => consumer.spawn(commands, ship, component),
            Self::EfficiencyProvider(provider) => {
                commands.entity(component).insert(*provider);
            }
        }
    }
}
