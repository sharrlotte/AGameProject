use bevy::prelude::*;

use crate::content::{
    block::{
        base::{Block, BlockDef, BlockOwner, ComponentOwner},
        component::{
            efficiency::{EfficiencyPlugin, EfficiencyProvider}, energy::{EnergyConsumerComponent, EnergyPlugin}, sprite::SpriteComponent, storage::StorageComponent, thruster::{ThrusterComponent, ThrusterPlugin}, turret::TurretComponent
        },
        turret::TurretPlugin,
    },
    team::Team,
};
pub struct ComponentPlugin;

impl Plugin for ComponentPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ThrusterPlugin, TurretPlugin, EnergyPlugin, EfficiencyPlugin))
            .add_systems(FixedUpdate, insert_added_block);
    }
}

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
            Self::Turret(turret) => turret.spawn(commands, component, def, team),
            Self::Thruster(thruster) => thruster.spawn(commands, component),
            Self::EnergyConsumer(consumer) => consumer.spawn(commands, component),
            Self::EfficiencyProvider(provider) => {
                commands.entity(component).insert(*provider);
            }
            Self::Storage(storage) => storage.spawn(commands, component),
        }
    }
}

fn insert_added_block(
    blocks: Query<(Entity, &Block, &BlockOwner), Added<Block>>,
    children: Query<&Children>,
    mut commands: Commands,
) {
    // child, ship, block
    let mut queue: Vec<(Entity, Entity, Entity)> = Vec::new();

    for (entity, _, owner) in &blocks {
        queue.push((entity, owner.id, entity));
    }

    while let Some((entity, ship, block)) = queue.pop() {
        commands
            .entity(entity)
            .insert((BlockOwner { id: ship }, ComponentOwner { id: block }));

        if let Ok(children) = children.get(entity) {
            for child in children.iter() {
                queue.push((child, ship, block));
            }
        }
    }
}
