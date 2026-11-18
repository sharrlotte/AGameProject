use bevy::prelude::*;

use crate::content::block::{BlockRegistry, DropEvent};
use crate::content::body::BlockGrid;
use crate::content::lifetime::Lifetime;
use crate::content::team::Team;

use super::Debris;

pub fn explode_expired_debris(
    mut commands: Commands,
    time: Res<Time>,
    registry: Res<BlockRegistry>,
    mut drops: MessageWriter<DropEvent>,
    mut debris: Query<(Entity, &mut Lifetime, &Team, &GlobalTransform, &BlockGrid), With<Debris>>,
) {
    for (entity, mut lifetime, team, transform, grid) in &mut debris {
        lifetime.tick(&time);

        if !lifetime.timer().is_finished() {
            continue;
        }

        if !team.is_player() {
            for block in grid.blocks() {
                let Some(def) = registry.get_symbol(&block.placement.block_type) else {
                    continue;
                };
                let position = transform
                    .transform_point(block.placement.position_pixel().extend(0.0))
                    .truncate();

                for drop in &def.drops {
                    drops.write(DropEvent {
                        position,
                        drop: drop.clone(),
                    });
                }
            }
        }

        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::block::{
        BakedShape, BlockDef, BlockPlacement, ItemDrop, Loot, Rot90, ShapeSpec,
    };
    use crate::content::damage::DamageEvent;

    fn def(block_type: &str) -> BlockDef {
        BlockDef {
            name: block_type.to_string(),
            block_type: block_type.to_string(),
            block_group: "structure".to_string(),
            mass: 5.0,
            health: 50.0,
            shape: BakedShape::try_from(&ShapeSpec::Square(1)).expect("test shape bakes"),
            cost: 1,
            features: Vec::new(),
            drops: vec![Loot::Item(ItemDrop {
                item_type: "steel".to_string(),
                amount: 2,
            })],
        }
    }

    fn app() -> App {
        let mut registry = BlockRegistry::default();
        registry.register("plate", def("plate"));

        let mut app = App::new();
        app.add_message::<DropEvent>()
            .add_message::<DamageEvent>()
            .insert_resource(registry)
            .insert_resource(Time::<()>::default())
            .add_systems(Update, explode_expired_debris);
        app
    }

    fn spawn(app: &mut App, team: Team) -> Entity {
        let mut grid = BlockGrid::new();
        {
            let registry = app.world().resource::<BlockRegistry>();
            grid.rebuild_from_layout(
                &[BlockPlacement::new(IVec2::ZERO, "plate", Rot90::R0)],
                registry,
            );
        }

        app.world_mut()
            .spawn((
                Debris,
                team,
                Transform::default(),
                Lifetime::seconds(0.0),
                grid,
            ))
            .id()
    }

    #[test]
    fn expired_non_player_debris_drops_loot_and_despawns() {
        let mut app = app();
        let debris = spawn(&mut app, Team::new(1));

        app.update();

        let events: Vec<DropEvent> = app
            .world_mut()
            .resource_mut::<Messages<DropEvent>>()
            .drain()
            .collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].drop, Loot::Item(ItemDrop {
            item_type: "steel".to_string(),
            amount: 2,
        }));
        assert!(app.world().get_entity(debris).is_err());
    }

    #[test]
    fn expired_player_debris_yields_nothing() {
        let mut app = app();
        let debris = spawn(&mut app, Team::new(0));

        app.update();

        let events: Vec<DropEvent> = app
            .world_mut()
            .resource_mut::<Messages<DropEvent>>()
            .drain()
            .collect();
        assert!(events.is_empty());
        assert!(app.world().get_entity(debris).is_err());
    }

    #[test]
    fn expiry_applies_no_damage() {
        let mut app = app();
        spawn(&mut app, Team::new(1));

        app.update();

        let events: Vec<DamageEvent> = app
            .world_mut()
            .resource_mut::<Messages<DamageEvent>>()
            .drain()
            .collect();
        assert!(events.is_empty());
    }
}
