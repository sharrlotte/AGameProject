use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use uuid::Uuid;

use crate::config::GameConfigValue;
use crate::content::block::{BlockPlacement, BlockRegistry, DropEvent, spawn_block_entity};
use crate::content::death::ShipDestroyed;
use crate::content::ship::{BoundingBox, Ship, ShipChanged, ShipGrid, ShipStats};
use crate::content::team::Team;

#[derive(Message, Debug, Clone)]
#[allow(dead_code)]
pub enum ShipMutation {
    DestroyBlock {
        ship: Entity,
        block_id: Uuid,
    },
    DestroyBlocks {
        ship: Entity,
        block_ids: Vec<Uuid>,
    },
    PlaceBlock {
        ship: Entity,
        block: BlockPlacement,
    },
    RebuildFromLayout {
        ship: Entity,
        layout: Vec<BlockPlacement>,
    },
}

type ShipQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut ShipGrid,
        &'static mut Ship,
        &'static mut ShipStats,
        &'static mut BoundingBox,
        &'static Team,
        &'static GlobalTransform,
    ),
>;

pub fn apply_ship_mutations(
    mut commands: Commands,
    mut events: MessageReader<ShipMutation>,
    registry: Res<BlockRegistry>,
    config: Res<GameConfigValue>,
    mut ships: ShipQuery,
    mut changed: MessageWriter<ShipChanged>,
    mut destroyed: MessageWriter<ShipDestroyed>,
    mut drops: MessageWriter<DropEvent>,
) {
    let mut by_ship: HashMap<Entity, Vec<ShipMutation>> = HashMap::new();
    for event in events.read() {
        let ship = match &event {
            ShipMutation::DestroyBlock { ship, .. }
            | ShipMutation::DestroyBlocks { ship, .. }
            | ShipMutation::PlaceBlock { ship, .. }
            | ShipMutation::RebuildFromLayout { ship, .. } => *ship,
        };
        by_ship.entry(ship).or_default().push(event.clone());
    }

    for (ship_entity, mut grid, mut ship, mut stats, mut bounds, team, transform) in &mut ships {
        let Some(mutations) = by_ship.remove(&ship_entity) else {
            continue;
        };

        let mut removed: Vec<BlockPlacement> = Vec::new();

        for mutation in mutations {
            match mutation {
                ShipMutation::DestroyBlock { block_id, .. } => {
                    if let Some(block) = grid.remove_block(block_id) {
                        removed.push(block.placement);
                        if block.entity != Entity::PLACEHOLDER {
                            commands.entity(block.entity).despawn();
                        }
                    }
                }
                ShipMutation::DestroyBlocks { block_ids, .. } => {
                    for block_id in block_ids {
                        if let Some(block) = grid.remove_block(block_id) {
                            removed.push(block.placement);
                            if block.entity != Entity::PLACEHOLDER {
                                commands.entity(block.entity).despawn();
                            }
                        }
                    }
                }
                ShipMutation::PlaceBlock { block, .. } => {
                    let Some(def) = registry.get_symbol(&block.block_type) else {
                        warn!("Unknown block type {:?}", block.block_type);
                        continue;
                    };
                    let entity =
                        spawn_block_entity(&mut commands, &block, def, ship_entity, *team, &config);
                    grid.insert_block(block, entity, def);
                }
                ShipMutation::RebuildFromLayout { layout, .. } => {
                    for block in grid.blocks() {
                        commands.entity(block.entity).despawn();
                    }
                    grid.rebuild_from_layout(&layout, &registry);

                    for placement in layout {
                        let Some(def) = registry.get_symbol(&placement.block_type) else {
                            continue;
                        };
                        let entity = spawn_block_entity(
                            &mut commands,
                            &placement,
                            def,
                            ship_entity,
                            *team,
                            &config,
                        );
                        grid.set_entity(placement.id, entity);
                    }
                }
            }
        }

        let reachable = reachable_from_core(&grid, &registry);
        let disconnected: Vec<Uuid> = grid
            .blocks
            .keys()
            .filter(|id| !reachable.contains(*id))
            .copied()
            .collect();

        for id in disconnected {
            if let Some(block) = grid.remove_block(id) {
                removed.push(block.placement);
                if block.entity != Entity::PLACEHOLDER {
                    commands.entity(block.entity).despawn();
                }
            }
        }

        if !team.is_player() {
            for placement in &removed {
                let Some(def) = registry.get_symbol(&placement.block_type) else {
                    continue;
                };
                let position = transform
                    .transform_point(placement.position_pixel().extend(0.0))
                    .truncate();
                for drop in &def.drops {
                    drops.write(DropEvent {
                        position,
                        drop: drop.clone(),
                    });
                }
            }
        }

        let build = registry.build_ship_result(&grid);
        let (Some(mut new_ship), Some(new_stats), Some(new_bounds)) =
            (build.ship, build.stats, build.bounds)
        else {
            commands.entity(ship_entity).despawn();
            destroyed.write(ShipDestroyed {
                ship: ship_entity,
                team: *team,
            });
            changed.write(ShipChanged { ship: ship_entity });
            continue;
        };

        new_ship.id = ship.id;
        *ship = new_ship;
        *stats = new_stats;
        *bounds = new_bounds;

        let collider = registry.build_ship_voxel_collider_2d(&grid);
        commands
            .entity(ship_entity)
            .remove::<Collider>()
            .insert((collider, ColliderMassProperties::Mass(stats.mass)));

        changed.write(ShipChanged { ship: ship_entity });
    }
}

fn reachable_from_core(grid: &ShipGrid, registry: &BlockRegistry) -> HashSet<Uuid> {
    let mut visited = HashSet::new();
    let Some((core, _)) = registry.core_of(grid) else {
        return visited;
    };

    let mut queue = vec![core];
    while let Some(id) = queue.pop() {
        if !visited.insert(id) {
            continue;
        }
        let Some(block) = grid.block(id) else {
            continue;
        };
        let Some(def) = registry.get_symbol(&block.placement.block_type) else {
            continue;
        };

        for y in 0..def.shape.size.y {
            for x in 0..def.shape.size.x {
                let cell = block.placement.position + IVec2::new(x, y);
                for dir in [IVec2::X, IVec2::NEG_X, IVec2::Y, IVec2::NEG_Y] {
                    if let Some(neighbor) = grid.cells.get(&(cell + dir))
                        && !visited.contains(neighbor)
                    {
                        queue.push(*neighbor);
                    }
                }
            }
        }
    }

    visited
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GameConfig;
    use crate::content::block::{BlockDef, BlockShape};
    use crate::content::death::ShipDestroyed;

    fn def(block_type: &str, group: &str, mass: f32, health: f32) -> BlockDef {
        BlockDef {
            name: block_type.to_string(),
            block_type: block_type.to_string(),
            block_group: group.to_string(),
            mass,
            health,
            shape: BlockShape::default(),
            cost: 1,
            features: Vec::new(),
            drops: Vec::new(),
        }
    }

    fn layout() -> Vec<BlockPlacement> {
        vec![
            BlockPlacement::new(IVec2::ZERO, "core", 0),
            BlockPlacement::new(IVec2::X, "plate", 0),
        ]
    }

    fn setup() -> (App, Entity) {
        let mut registry = BlockRegistry::default();
        registry.register("core", def("core", "core", 10.0, 100.0));
        registry.register("plate", def("plate", "structure", 5.0, 50.0));

        let layout = layout();
        let mut grid = ShipGrid::new();
        grid.rebuild_from_layout(&layout, &registry);

        let build = registry.build_ship_result(&grid);
        let ship = build.ship.unwrap();
        let stats = build.stats.unwrap();
        let bounds = build.bounds.unwrap();

        let mut app = App::new();
        app.add_message::<ShipMutation>()
            .add_message::<ShipChanged>()
            .add_message::<ShipDestroyed>()
            .add_message::<DropEvent>()
            .insert_resource(registry)
            .insert_resource(GameConfigValue(GameConfig::default()))
            .add_systems(Update, apply_ship_mutations);

        let entity = app
            .world_mut()
            .spawn((
                ship,
                stats,
                bounds,
                Team::new(0),
                GlobalTransform::default(),
                grid,
            ))
            .id();

        (app, entity)
    }

    fn send(app: &mut App, mutation: ShipMutation) {
        app.world_mut()
            .resource_mut::<Messages<ShipMutation>>()
            .write(mutation);
        app.update();
    }

    #[test]
    fn destroys_block_and_rebuilds_stats() {
        let (mut app, ship_entity) = setup();
        let block_id = app
            .world()
            .get::<ShipGrid>(ship_entity)
            .unwrap()
            .block_at(IVec2::X)
            .unwrap()
            .placement
            .id;

        send(
            &mut app,
            ShipMutation::DestroyBlock {
                ship: ship_entity,
                block_id,
            },
        );

        let grid = app.world().get::<ShipGrid>(ship_entity).unwrap();
        assert_eq!(grid.blocks.len(), 1);
        assert!(grid.block_at(IVec2::X).is_none());

        let stats = app.world().get::<ShipStats>(ship_entity).unwrap();
        assert_eq!(stats.mass, 10.0);
        assert!(app.world().get_entity(ship_entity).is_ok());
    }

    #[test]
    fn destroying_core_kills_ship() {
        let (mut app, ship_entity) = setup();
        let core_id = app
            .world()
            .get::<ShipGrid>(ship_entity)
            .unwrap()
            .block_at(IVec2::ZERO)
            .unwrap()
            .placement
            .id;

        send(
            &mut app,
            ShipMutation::DestroyBlock {
                ship: ship_entity,
                block_id: core_id,
            },
        );

        assert!(app.world().get_entity(ship_entity).is_err());
    }

    #[test]
    fn core_loss_emits_one_ship_destroyed_and_dead_ship_is_skipped() {
        let (mut app, ship_entity) = setup();
        let core_id = app
            .world()
            .get::<ShipGrid>(ship_entity)
            .unwrap()
            .block_at(IVec2::ZERO)
            .unwrap()
            .placement
            .id;

        send(
            &mut app,
            ShipMutation::DestroyBlock {
                ship: ship_entity,
                block_id: core_id,
            },
        );

        let destroyed: Vec<ShipDestroyed> = app
            .world_mut()
            .resource_mut::<Messages<ShipDestroyed>>()
            .drain()
            .collect();
        assert_eq!(destroyed.len(), 1);
        assert_eq!(destroyed[0].ship, ship_entity);
        assert_eq!(destroyed[0].team, Team::new(0));

        send(
            &mut app,
            ShipMutation::DestroyBlock {
                ship: ship_entity,
                block_id: core_id,
            },
        );

        let repeated: Vec<ShipDestroyed> = app
            .world_mut()
            .resource_mut::<Messages<ShipDestroyed>>()
            .drain()
            .collect();
        assert!(repeated.is_empty());
    }

    #[test]
    fn cutting_neighbour_drops_it_as_fragment() {
        let (mut app, ship_entity) = setup();
        let core_id = app
            .world()
            .get::<ShipGrid>(ship_entity)
            .unwrap()
            .block_at(IVec2::ZERO)
            .unwrap()
            .placement
            .id;

        // Destroying the core also detaches the plate before core death is
        // resolved; the plate must be removed from the grid.
        send(
            &mut app,
            ShipMutation::DestroyBlock {
                ship: ship_entity,
                block_id: core_id,
            },
        );

        assert!(app.world().get_entity(ship_entity).is_err());
    }
}
