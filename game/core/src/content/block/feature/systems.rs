use crate::content::block::{
    components::{Block, BlockOwner, ComponentOwner},
    feature::{
        efficiency::EfficiencyPlugin, energy::EnergyPlugin, thruster::ThrusterPlugin,
        turret::TurretPlugin,
    },
};
use crate::content::team::Team;
use bevy::prelude::*;

pub struct ComponentPlugin;

impl Plugin for ComponentPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ThrusterPlugin, TurretPlugin, EnergyPlugin, EfficiencyPlugin))
            .add_systems(FixedUpdate, insert_added_block);
    }
}

pub fn insert_added_block(
    blocks: Query<(Entity, &Block, &BlockOwner, &Team), Added<Block>>,
    children: Query<&Children>,
    mut commands: Commands,
) {
    // child, ship, block
    let mut queue: Vec<(Entity, Entity, Entity, Team)> = Vec::new();

    for (entity, _, owner, team) in &blocks {
        queue.push((entity, owner.id, entity, *team));
    }

    while let Some((entity, ship, block, team)) = queue.pop() {
        commands.entity(entity).insert((
            BlockOwner { id: ship },
            ComponentOwner { id: block },
            team.clone(),
        ));

        if let Ok(children) = children.get(entity) {
            for child in children.iter() {
                queue.push((child, ship, block, team));
            }
        }
    }
}
