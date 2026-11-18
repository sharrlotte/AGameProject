use std::collections::HashMap;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use uuid::Uuid;

use crate::config::GameConfigValue;
use crate::content::block::{BlockPlacement, BlockRegistry, DropEvent};
use crate::content::body::debris::{
    component_centroid, component_mass, debris_transform, debris_velocity, detach_component,
    next_lifetime, set_block_owner, shift_block_transform, spawn_debris_body,
};
use crate::content::body::BlockGrid;
use crate::content::death::ShipDestroyed;
use crate::content::lifetime::Lifetime;
use crate::content::ship::{BoundingBox, Ship, ShipChanged, ShipStats};
use crate::content::team::Team;

#[derive(Message, Debug, Clone)]
pub enum BodyMutation {
    DestroyBlock { body: Entity, block_id: Uuid },
}

type BodyQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut BlockGrid,
        Option<&'static mut Ship>,
        Option<&'static mut ShipStats>,
        Option<&'static mut BoundingBox>,
        &'static Team,
        &'static GlobalTransform,
        &'static Velocity,
        Option<&'static Lifetime>,
    ),
>;

pub fn apply_body_mutations(
    mut commands: Commands,
    mut events: MessageReader<BodyMutation>,
    registry: Res<BlockRegistry>,
    config: Res<GameConfigValue>,
    mut bodies: BodyQuery,
    mut changed: MessageWriter<ShipChanged>,
    mut destroyed: MessageWriter<ShipDestroyed>,
    mut drops: MessageWriter<DropEvent>,
) {
    let mut by_body: HashMap<Entity, Vec<BodyMutation>> = HashMap::new();
    for event in events.read() {
        let BodyMutation::DestroyBlock { body, .. } = event;
        by_body.entry(*body).or_default().push(event.clone());
    }

    for (entity, mut grid, ship, stats, bounds, team, transform, velocity, lifetime) in &mut bodies {
        let Some(mutations) = by_body.remove(&entity) else {
            continue;
        };

        let mut removed: Vec<(BlockPlacement, Vec2)> = Vec::new();

        for mutation in mutations {
            let BodyMutation::DestroyBlock { block_id, .. } = mutation;
            if let Some(block) = grid.remove_block(block_id) {
                let position = transform
                    .transform_point(block.placement.position_pixel().extend(0.0))
                    .truncate();
                removed.push((block.placement, position));

                if block.entity != Entity::PLACEHOLDER {
                    commands.entity(block.entity).despawn();
                }
            }
        }

        if !team.is_player() {
            emit_drops(&mut drops, &registry, &removed);
        }

        match ship {
            Some(mut ship) => {
                apply_ship_mutation(
                    &mut commands,
                    &mut grid,
                    &mut ship,
                    stats,
                    bounds,
                    *team,
                    transform,
                    velocity,
                    &entity,
                    &registry,
                    &config,
                    &mut changed,
                    &mut destroyed,
                );
            }
            None => split_debris(
                &mut commands,
                &mut grid,
                lifetime,
                *team,
                transform,
                velocity,
                &entity,
                &registry,
                &config,
            ),
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_ship_mutation(
    commands: &mut Commands,
    grid: &mut BlockGrid,
    ship: &mut Ship,
    stats: Option<Mut<ShipStats>>,
    bounds: Option<Mut<BoundingBox>>,
    team: Team,
    transform: &GlobalTransform,
    velocity: &Velocity,
    entity: &Entity,
    registry: &BlockRegistry,
    config: &GameConfigValue,
    changed: &mut MessageWriter<ShipChanged>,
    destroyed: &mut MessageWriter<ShipDestroyed>,
) {
    let Some((core_id, _)) = registry.core_of(grid) else {
        for component in grid.components(registry) {
            spawn_debris_from_component(
                commands,
                grid,
                &component,
                registry,
                transform,
                velocity,
                team,
                next_lifetime(None, config),
                config,
            );
        }

        commands.entity(*entity).despawn();
        destroyed.write(ShipDestroyed {
            ship: *entity,
            team,
        });
        return;
    };

    for component in grid.components(registry) {
        if component.contains(&core_id) {
            continue;
        }

        spawn_debris_from_component(
            commands,
            grid,
            &component,
            registry,
            transform,
            velocity,
            team,
            next_lifetime(None, config),
            config,
        );
    }

    rebuild_ship(commands, grid, ship, stats, bounds, entity, registry, changed);
}

fn rebuild_ship(
    commands: &mut Commands,
    grid: &BlockGrid,
    ship: &mut Ship,
    stats: Option<Mut<ShipStats>>,
    bounds: Option<Mut<BoundingBox>>,
    entity: &Entity,
    registry: &BlockRegistry,
    changed: &mut MessageWriter<ShipChanged>,
) {
    let Some(mut build) = registry.build_grid_result(grid) else {
        return;
    };

    build.ship.id = ship.id;
    *ship = build.ship;
    let mass = build.stats.mass;
    if let Some(mut stats) = stats {
        *stats = build.stats;
    }
    if let Some(mut bounds) = bounds {
        *bounds = build.bounds;
    }

    let collider = registry.build_voxel_collider_2d(grid);
    commands
        .entity(*entity)
        .remove::<Collider>()
        .insert((collider, ColliderMassProperties::Mass(mass)));

    changed.write(ShipChanged { ship: *entity });
}

#[allow(clippy::too_many_arguments)]
fn split_debris(
    commands: &mut Commands,
    grid: &mut BlockGrid,
    lifetime: Option<&Lifetime>,
    team: Team,
    transform: &GlobalTransform,
    velocity: &Velocity,
    entity: &Entity,
    registry: &BlockRegistry,
    config: &GameConfigValue,
) {
    let components = grid.components(registry);

    let Some(keep) = keeper_component(&components) else {
        commands.entity(*entity).despawn();
        return;
    };

    let remaining = next_lifetime(lifetime, config);

    for (index, component) in components.iter().enumerate() {
        if index == keep {
            continue;
        }

        spawn_debris_from_component(
            commands,
            grid,
            component,
            registry,
            transform,
            velocity,
            team,
            remaining,
            config,
        );
    }

    let collider = registry.build_voxel_collider_2d(grid);
    let mass = component_mass(grid, registry);
    commands
        .entity(*entity)
        .remove::<Collider>()
        .insert((collider, ColliderMassProperties::Mass(mass)));
}

fn keeper_component(components: &[Vec<Uuid>]) -> Option<usize> {
    components
        .iter()
        .enumerate()
        .max_by(|(_, left), (_, right)| {
            left.len()
                .cmp(&right.len())
                .then_with(|| right.iter().min().cmp(&left.iter().min()))
        })
        .map(|(index, _)| index)
}

#[allow(clippy::too_many_arguments)]
fn spawn_debris_from_component(
    commands: &mut Commands,
    source: &mut BlockGrid,
    ids: &[Uuid],
    registry: &BlockRegistry,
    parent: &GlobalTransform,
    velocity: &Velocity,
    team: Team,
    lifetime_secs: f32,
    config: &GameConfigValue,
) {
    let centroid = component_centroid(source, ids);

    let Some(detached) = detach_component(source, ids, registry) else {
        return;
    };

    let transform = debris_transform(parent, detached.anchor);
    let inherited = debris_velocity(velocity, parent, centroid);
    let collider = registry.build_voxel_collider_2d(&detached.grid);
    let mass = component_mass(&detached.grid, registry);

    let body = spawn_debris_body(
        commands,
        detached.grid,
        transform,
        inherited,
        team,
        lifetime_secs,
        collider,
        mass,
        config,
    );

    let anchor = detached.anchor;
    for block in detached.blocks {
        if block == Entity::PLACEHOLDER {
            continue;
        }

        commands.entity(block).queue(move |mut entity: EntityWorldMut| {
            if let Some(mut transform) = entity.get_mut::<Transform>() {
                shift_block_transform(&mut transform, anchor);
            }
        });
        set_block_owner(commands, block, body);
        commands.entity(body).add_child(block);
    }
}

fn emit_drops(
    drops: &mut MessageWriter<DropEvent>,
    registry: &BlockRegistry,
    removed: &[(BlockPlacement, Vec2)],
) {
    for (placement, position) in removed {
        let Some(def) = registry.get_symbol(&placement.block_type) else {
            continue;
        };

        for drop in &def.drops {
            drops.write(DropEvent {
                position: *position,
                drop: drop.clone(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GameConfig;
    use crate::constant::GRID_SIZE;
    use crate::content::block::{BakedShape, BlockDef, Rot90, ShapeSpec};
    use crate::content::body::Debris;

    fn def(block_type: &str, group: &str, mass: f32, health: f32) -> BlockDef {
        BlockDef {
            name: block_type.to_string(),
            block_type: block_type.to_string(),
            block_group: group.to_string(),
            mass,
            health,
            shape: BakedShape::try_from(&ShapeSpec::Square(1)).expect("test shape bakes"),
            cost: 1,
            features: Vec::new(),
            drops: Vec::new(),
        }
    }

    fn registry() -> BlockRegistry {
        let mut registry = BlockRegistry::default();
        registry.register("core", def("core", "core", 10.0, 100.0));
        registry.register("plate", def("plate", "structure", 5.0, 50.0));
        registry
    }

    fn app() -> App {
        let mut app = App::new();
        app.add_message::<BodyMutation>()
            .add_message::<ShipChanged>()
            .add_message::<ShipDestroyed>()
            .add_message::<DropEvent>()
            .insert_resource(registry())
            .insert_resource(GameConfigValue(GameConfig::default()))
            .add_systems(Update, apply_body_mutations);
        app
    }

    fn spawn_ship(app: &mut App, layout: &[BlockPlacement]) -> Entity {
        let mut grid = BlockGrid::new();
        {
            let registry = app.world().resource::<BlockRegistry>();
            grid.rebuild_from_layout(layout, registry);
        }

        let (ship, stats, bounds) = {
            let registry = app.world().resource::<BlockRegistry>();
            let build = registry.build_grid_result(&grid).expect("valid ship build");
            (build.ship, build.stats, build.bounds)
        };

        app.world_mut()
            .spawn((
                ship,
                stats,
                bounds,
                Team::new(0),
                Transform::default(),
                GlobalTransform::default(),
                Velocity::zero(),
                grid,
            ))
            .id()
    }

    fn spawn_debris(app: &mut App, layout: &[BlockPlacement], lifetime: f32) -> Entity {
        let mut grid = BlockGrid::new();
        {
            let registry = app.world().resource::<BlockRegistry>();
            grid.rebuild_from_layout(layout, registry);
        }

        app.world_mut()
            .spawn((
                Debris,
                Team::new(0),
                Transform::default(),
                GlobalTransform::default(),
                Velocity::zero(),
                Lifetime::seconds(lifetime),
                grid,
            ))
            .id()
    }

    fn send(app: &mut App, body: Entity, block_id: Uuid) {
        app.world_mut()
            .resource_mut::<Messages<BodyMutation>>()
            .write(BodyMutation::DestroyBlock { body, block_id });
        app.update();
    }

    fn block_id(app: &App, body: Entity, cell: IVec2) -> Uuid {
        app.world()
            .get::<BlockGrid>(body)
            .expect("body has a grid")
            .block_at(cell)
            .expect("block at cell")
            .placement
            .id
    }

    fn debris_bodies(app: &mut App) -> Vec<(Entity, usize)> {
        let world = app.world_mut();
        let mut query = world.query::<(Entity, &Debris, &BlockGrid)>();
        query
            .iter(world)
            .map(|(entity, _, grid)| (entity, grid.blocks.len()))
            .collect()
    }

    fn chain() -> Vec<BlockPlacement> {
        vec![
            BlockPlacement::new(IVec2::ZERO, "core", Rot90::R0),
            BlockPlacement::new(IVec2::X, "plate", Rot90::R0),
            BlockPlacement::new(IVec2::new(2, 0), "plate", Rot90::R0),
        ]
    }

    fn debris_chain() -> Vec<BlockPlacement> {
        vec![
            BlockPlacement::new(IVec2::ZERO, "plate", Rot90::R0),
            BlockPlacement::new(IVec2::X, "plate", Rot90::R0),
            BlockPlacement::new(IVec2::new(2, 0), "plate", Rot90::R0),
        ]
    }

    #[test]
    fn destroys_block_and_rebuilds_stats() {
        let mut app = app();
        let ship = spawn_ship(&mut app, &chain());
        let bridge = block_id(&app, ship, IVec2::X);

        send(&mut app, ship, bridge);

        let grid = app.world().get::<BlockGrid>(ship).unwrap();
        assert_eq!(grid.blocks.len(), 1);
        assert!(grid.block_at(IVec2::X).is_none());

        let stats = app.world().get::<ShipStats>(ship).unwrap();
        assert_eq!(stats.mass, 10.0);

        let collider = app.world().get::<Collider>(ship).unwrap();
        assert_eq!(collider.as_compound().expect("compound").shapes().len(), 1);

        let bodies = debris_bodies(&mut app);
        assert_eq!(bodies.len(), 1);
        assert_eq!(bodies[0].1, 1);
    }

    #[test]
    fn destroying_core_kills_ship_and_wrecks_survivors() {
        let mut app = app();
        let ship = spawn_ship(
            &mut app,
            &[
                BlockPlacement::new(IVec2::ZERO, "core", Rot90::R0),
                BlockPlacement::new(IVec2::X, "plate", Rot90::R0),
            ],
        );
        let core = block_id(&app, ship, IVec2::ZERO);

        send(&mut app, ship, core);

        assert!(app.world().get_entity(ship).is_err());
        assert_eq!(debris_bodies(&mut app).len(), 1);
    }

    #[test]
    fn core_loss_emits_one_ship_destroyed_and_dead_ship_is_skipped() {
        let mut app = app();
        let ship = spawn_ship(
            &mut app,
            &[
                BlockPlacement::new(IVec2::ZERO, "core", Rot90::R0),
                BlockPlacement::new(IVec2::X, "plate", Rot90::R0),
            ],
        );
        let core = block_id(&app, ship, IVec2::ZERO);

        send(&mut app, ship, core);

        let destroyed: Vec<ShipDestroyed> = app
            .world_mut()
            .resource_mut::<Messages<ShipDestroyed>>()
            .drain()
            .collect();
        assert_eq!(destroyed.len(), 1);
        assert_eq!(destroyed[0].ship, ship);
        assert_eq!(destroyed[0].team, Team::new(0));

        send(&mut app, ship, core);

        let repeated: Vec<ShipDestroyed> = app
            .world_mut()
            .resource_mut::<Messages<ShipDestroyed>>()
            .drain()
            .collect();
        assert!(repeated.is_empty());
    }

    #[test]
    fn detaching_mid_ship_section_creates_debris() {
        let mut app = app();
        let ship = spawn_ship(&mut app, &chain());
        let bridge = block_id(&app, ship, IVec2::X);

        send(&mut app, ship, bridge);

        assert_eq!(app.world().get::<BlockGrid>(ship).unwrap().blocks.len(), 1);
        let bodies = debris_bodies(&mut app);
        assert_eq!(bodies.len(), 1);
        assert_eq!(bodies[0].1, 1);
    }

    #[test]
    fn detached_debris_inherits_parent_momentum() {
        let mut app = app();
        let ship = spawn_ship(&mut app, &chain());
        let bridge = block_id(&app, ship, IVec2::X);
        *app.world_mut().get_mut::<Velocity>(ship).unwrap() = Velocity {
            linear: Vec2::new(3.0, 0.0),
            angular: 1.0,
        };

        send(&mut app, ship, bridge);

        let (debris, _) = debris_bodies(&mut app)
            .into_iter()
            .next()
            .expect("debris body");
        let velocity = app.world().get::<Velocity>(debris).unwrap();
        assert!((velocity.linear.x - (3.0 - 0.5 * GRID_SIZE)).abs() < 1e-4);
        assert!((velocity.linear.y - 2.5 * GRID_SIZE).abs() < 1e-3);
        assert!((velocity.angular - 1.0).abs() < 1e-4);
    }

    #[test]
    fn detached_debris_preserves_world_centers_under_rotation() {
        let mut app = app();
        let ship = spawn_ship(&mut app, &chain());
        let transform = Transform::from_xyz(100.0, 50.0, 0.0)
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2));
        *app.world_mut().get_mut::<Transform>(ship).unwrap() = transform;
        *app.world_mut().get_mut::<GlobalTransform>(ship).unwrap() = GlobalTransform::from(transform);

        let tail_center = {
            let grid = app.world().get::<BlockGrid>(ship).unwrap();
            let placement = &grid.block_at(IVec2::new(2, 0)).unwrap().placement;
            placement.position.as_vec2() * GRID_SIZE + Vec2::splat(GRID_SIZE * 0.5)
        };
        let before = GlobalTransform::from(transform).transform_point(tail_center.extend(0.0));

        let bridge = block_id(&app, ship, IVec2::X);
        send(&mut app, ship, bridge);

        let (debris, _) = debris_bodies(&mut app)
            .into_iter()
            .next()
            .expect("debris body");
        let debris_transform = *app.world().get::<Transform>(debris).unwrap();
        let debris_grid = app.world().get::<BlockGrid>(debris).unwrap();
        let tail = debris_grid.block_at(IVec2::ZERO).expect("tail in debris grid");
        let center = tail.placement.position.as_vec2() * GRID_SIZE + Vec2::splat(GRID_SIZE * 0.5);
        let after = debris_transform.transform_point(center.extend(0.0));

        assert!(
            (after - before).length() < 1e-3,
            "before {before:?}, after {after:?}"
        );
    }

    #[test]
    fn debris_split_creates_two_bodies_and_keeps_remaining_lifetime() {
        let mut app = app();
        let debris = spawn_debris(&mut app, &debris_chain(), 5.0);
        let bridge = block_id(&app, debris, IVec2::X);

        send(&mut app, debris, bridge);

        let bodies = debris_bodies(&mut app);
        assert_eq!(bodies.len(), 2);
        assert_eq!(bodies.iter().map(|(_, blocks)| blocks).sum::<usize>(), 2);

        for (entity, _) in bodies {
            let lifetime = app.world().get::<Lifetime>(entity).unwrap();
            let remaining = lifetime.timer().remaining_secs();
            assert!(remaining > 0.0 && remaining <= 5.0, "{remaining}");
        }
    }

    #[test]
    fn debris_shrinks_without_splitting() {
        let mut app = app();
        let debris = spawn_debris(
            &mut app,
            &[
                BlockPlacement::new(IVec2::ZERO, "plate", Rot90::R0),
                BlockPlacement::new(IVec2::X, "plate", Rot90::R0),
            ],
            5.0,
        );
        let tail = block_id(&app, debris, IVec2::X);

        send(&mut app, debris, tail);

        assert_eq!(debris_bodies(&mut app), vec![(debris, 1)]);
        let collider = app.world().get::<Collider>(debris).unwrap();
        assert_eq!(collider.as_compound().expect("compound").shapes().len(), 1);
    }

    #[test]
    fn last_debris_block_despawns_body() {
        let mut app = app();
        let debris = spawn_debris(
            &mut app,
            &[BlockPlacement::new(IVec2::ZERO, "plate", Rot90::R0)],
            5.0,
        );
        let only = block_id(&app, debris, IVec2::ZERO);

        send(&mut app, debris, only);

        assert!(app.world().get_entity(debris).is_err());
        assert!(debris_bodies(&mut app).is_empty());
    }

    #[test]
    fn debris_mutation_leaves_ship_stats_untouched() {
        let mut app = app();
        let ship = spawn_ship(
            &mut app,
            &[
                BlockPlacement::new(IVec2::ZERO, "core", Rot90::R0),
                BlockPlacement::new(IVec2::X, "plate", Rot90::R0),
            ],
        );
        let debris = spawn_debris(
            &mut app,
            &[
                BlockPlacement::new(IVec2::ZERO, "plate", Rot90::R0),
                BlockPlacement::new(IVec2::X, "plate", Rot90::R0),
            ],
            5.0,
        );

        let stats_before = app.world().get::<ShipStats>(ship).unwrap().clone();
        let bounds_before = app.world().get::<BoundingBox>(ship).unwrap().clone();

        let tail = block_id(&app, debris, IVec2::X);
        send(&mut app, debris, tail);

        let stats_after = app.world().get::<ShipStats>(ship).unwrap();
        assert_eq!(stats_after.mass, stats_before.mass);
        assert_eq!(stats_after.health, stats_before.health);
        assert_eq!(stats_after.max_health, stats_before.max_health);
        assert_eq!(stats_after.cargo_used, stats_before.cargo_used);
        assert_eq!(
            app.world().get::<BoundingBox>(ship).unwrap().size,
            bounds_before.size
        );
        let changed: Vec<ShipChanged> = app
            .world_mut()
            .resource_mut::<Messages<ShipChanged>>()
            .drain()
            .collect();
        assert!(changed.is_empty());
    }
}
