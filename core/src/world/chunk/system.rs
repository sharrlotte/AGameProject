use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use crate::constant::{CHUNK_SIZE, DESPAWN_RADIUS, LOAD_RADIUS};
use crate::config::GameConfigValue;
use crate::content::block::BlockRegistry;
use crate::content::body::{BlockGrid, Debris};
use crate::content::ship::Ship;
use crate::content::team::Team;
use crate::save::world::component::WorldContext;
use crate::save::world::snapshot::{BodySnapshot, DebrisSnapshot, ShipSnapshot, apply_bodies};
use crate::save::world::store::WorldStore;
use crate::world::chunk::component::{
    ChunkCoord, ChunkData, ChunkStreamError, StreamingTimer, StreamSnapshot,
};

type Captured<T> = (ChunkCoord, T, Vec<Entity>, Entity);
type Despawned = Vec<(Entity, Vec<Entity>)>;

pub fn stream_chunks(world: &mut World) {
    if !stream_interval_elapsed(world) {
        return;
    }

    if world.get_resource::<WorldContext>().is_none() {
        return;
    }

    let player_positions = collect_player_positions(world);
    if player_positions.is_empty() {
        return;
    }

    offload_out_of_range(world, &player_positions);
    load_in_range(world, &player_positions);
}

pub fn on_chunk_stream_error(mut errors: MessageReader<ChunkStreamError>) {
    for error in errors.read() {
        error!("Chunk stream error at {:?}: {}", error.coord, error.detail);
    }
}

fn stream_interval_elapsed(world: &mut World) -> bool {
    let Some(time) = world.get_resource::<Time>() else {
        return false;
    };
    let delta = time.delta();
    let Some(mut timer) = world.get_resource_mut::<StreamingTimer>() else {
        return false;
    };
    timer.0.tick(delta).just_finished()
}

fn min_dist_sq(pos: Vec2, players: &[Vec2]) -> f32 {
    players
        .iter()
        .map(|p| p.distance_squared(pos))
        .fold(f32::INFINITY, f32::min)
}

fn store(world: &World) -> Option<&dyn WorldStore> {
    world.get_resource::<WorldContext>().map(WorldContext::store)
}

fn emit_chunk_error(world: &mut World, coord: ChunkCoord, detail: String) {
    world.write_message(ChunkStreamError { coord, detail });
}

fn collect_player_positions(world: &mut World) -> Vec<Vec2> {
    let mut positions = Vec::new();
    let mut query = world.query_filtered::<(&Transform, &Team), With<Ship>>();
    for (transform, team) in query.iter(world) {
        if team.is_player() {
            positions.push(transform.translation.truncate());
        }
    }
    positions
}

fn collect_body_blocks(world: &World, entity: Entity) -> Vec<Entity> {
    world
        .get::<BlockGrid>(entity)
        .map(|grid| {
            grid.blocks()
                .filter(|b| b.entity != Entity::PLACEHOLDER)
                .map(|b| b.entity)
                .collect()
        })
        .unwrap_or_default()
}

fn collect_ship_candidates(world: &mut World, players: &[Vec2]) -> Vec<(Entity, Vec2)> {
    let despawn_radius_sq = DESPAWN_RADIUS * DESPAWN_RADIUS;
    let mut candidates = Vec::new();
    let mut query = world.query_filtered::<(Entity, &Transform, &Team), With<Ship>>();
    for (entity, transform, team) in query.iter(world) {
        if team.is_player() {
            continue;
        }
        let pos = transform.translation.truncate();
        if min_dist_sq(pos, players) > despawn_radius_sq {
            candidates.push((entity, pos));
        }
    }
    candidates
}

fn collect_debris_candidates(world: &mut World, players: &[Vec2]) -> Vec<(Entity, Vec2)> {
    let despawn_radius_sq = DESPAWN_RADIUS * DESPAWN_RADIUS;
    let mut candidates = Vec::new();
    let mut query = world.query_filtered::<(Entity, &Transform), With<Debris>>();
    for (entity, transform) in query.iter(world) {
        let pos = transform.translation.truncate();
        if min_dist_sq(pos, players) > despawn_radius_sq {
            candidates.push((entity, pos));
        }
    }
    candidates
}

fn capture_out_of_range<T: StreamSnapshot>(
    world: &World,
    candidates: &[(Entity, Vec2)],
) -> Vec<Captured<T>> {
    let mut captured = Vec::new();
    for &(entity, pos) in candidates {
        if let Some(snapshot) = T::capture(world, world.entity(entity)) {
            let coord = ChunkCoord::from_world(pos, CHUNK_SIZE);
            let blocks = collect_body_blocks(world, entity);
            captured.push((coord, snapshot, blocks, entity));
        }
    }
    captured
}

fn offload_out_of_range(world: &mut World, players: &[Vec2]) {
    let ships = collect_ship_candidates(world, players);
    let debris = collect_debris_candidates(world, players);
    let captured_ships = capture_out_of_range::<ShipSnapshot>(world, &ships);
    let captured_debris = capture_out_of_range::<DebrisSnapshot>(world, &debris);

    let mut updates: HashMap<ChunkCoord, (ChunkData, Despawned)> = HashMap::new();
    for (coord, snapshot, blocks, entity) in captured_ships {
        let entry = updates.entry(coord).or_default();
        snapshot.push_into(&mut entry.0);
        entry.1.push((entity, blocks));
    }
    for (coord, snapshot, blocks, entity) in captured_debris {
        let entry = updates.entry(coord).or_default();
        snapshot.push_into(&mut entry.0);
        entry.1.push((entity, blocks));
    }

    let mut errors = Vec::new();
    let mut persisted = Vec::new();
    for (coord, (new_data, entities)) in updates {
        let mut saved = false;
        {
            let Some(store) = store(world) else {
                continue;
            };
            let mut data = match store.load_chunk(coord) {
                Ok(data) => data.unwrap_or_default(),
                Err(e) => {
                    errors.push((coord, format!("load failed: {e}")));
                    continue;
                }
            };
            data.enemies.extend(new_data.enemies);
            data.debris.extend(new_data.debris);
            match store.save_chunk(coord, &data) {
                Ok(()) => saved = true,
                Err(e) => errors.push((coord, format!("save failed: {e}"))),
            }
        }
        if saved {
            persisted.extend(entities);
        }
    }

    for (coord, detail) in errors {
        emit_chunk_error(world, coord, detail);
    }
    despawn_persisted(world, &persisted);
}

fn despawn_persisted(world: &mut World, persisted: &Despawned) {
    for (entity, blocks) in persisted {
        for &block in blocks {
            if world.get_entity(block).is_ok() {
                world.despawn(block);
            }
        }
        if world.get_entity(*entity).is_ok() {
            world.despawn(*entity);
        }
    }
}

fn load_in_range(world: &mut World, players: &[Vec2]) {
    let mut neighbor_chunks = HashSet::new();
    for &player_pos in players {
        let center = ChunkCoord::from_world(player_pos, CHUNK_SIZE);
        neighbor_chunks.extend(center.neighborhood_3x3());
    }

    let load_radius_sq = LOAD_RADIUS * LOAD_RADIUS;
    let mut to_restore: Vec<BodySnapshot> = Vec::new();
    let mut errors = Vec::new();

    for coord in neighbor_chunks {
        let mut data = {
            let Some(store) = store(world) else {
                continue;
            };
            match store.load_chunk(coord) {
                Ok(Some(data)) => data,
                Ok(None) => continue,
                Err(e) => {
                    errors.push((coord, format!("load failed: {e}")));
                    continue;
                }
            }
        };

        let mut modified = false;

        let mut remaining_enemies = Vec::new();
        for snapshot in data.enemies {
            let pos = Vec2::new(snapshot.transform.x, snapshot.transform.y);
            if min_dist_sq(pos, players) <= load_radius_sq {
                to_restore.push(snapshot.into_body_snapshot());
                modified = true;
            } else {
                remaining_enemies.push(snapshot);
            }
        }
        data.enemies = remaining_enemies;

        let mut remaining_debris = Vec::new();
        for snapshot in data.debris {
            let pos = Vec2::new(snapshot.transform.x, snapshot.transform.y);
            if min_dist_sq(pos, players) <= load_radius_sq {
                to_restore.push(snapshot.into_body_snapshot());
                modified = true;
            } else {
                remaining_debris.push(snapshot);
            }
        }
        data.debris = remaining_debris;

        if modified {
            let Some(store) = store(world) else {
                continue;
            };
            if let Err(e) = store.save_chunk(coord, &data) {
                errors.push((coord, format!("update failed: {e}")));
            }
        }
    }

    for (coord, detail) in errors {
        emit_chunk_error(world, coord, detail);
    }

    if to_restore.is_empty() {
        return;
    }

    let error_coord = match &to_restore[0] {
        BodySnapshot::Ship(snapshot) => {
            ChunkCoord::from_world(Vec2::new(snapshot.transform.x, snapshot.transform.y), CHUNK_SIZE)
        }
        BodySnapshot::Debris(snapshot) => {
            ChunkCoord::from_world(Vec2::new(snapshot.transform.x, snapshot.transform.y), CHUNK_SIZE)
        }
    };
    let Some(game_config) = world.get_resource::<GameConfigValue>().cloned() else {
        return;
    };

    let mut restore_error = None;
    world.resource_scope(|world, registry: Mut<BlockRegistry>| {
        let mut commands = world.commands();
        if let Err(e) = apply_bodies(&mut commands, to_restore, &registry, &game_config) {
            restore_error = Some(format!("restore failed: {e}"));
        }
    });
    world.flush();

    if let Some(detail) = restore_error {
        emit_chunk_error(world, error_coord, detail);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    use crate::constant::STREAM_INTERVAL_SECS;
    use crate::content::block::{BakedShape, BlockDef, BlockPlacement, Rot90, ShapeSpec};
    use crate::content::lifetime::Lifetime;
    use crate::save::world::snapshot::{BlockSnapshot, TransformSave, VelocitySave};

    fn test_registry() -> BlockRegistry {
        let mut registry = BlockRegistry::default();
        registry.register(
            "core",
            BlockDef {
                name: "Core".to_string(),
                block_type: "core".to_string(),
                block_group: "core".to_string(),
                mass: 100.0,
                health: 500.0,
                shape: BakedShape::try_from(&ShapeSpec::Square(1)).expect("bake shape"),
                cost: 0,
                features: Vec::new(),
                drops: Vec::new(),
            },
        );
        registry
    }

    fn build_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Time>();
        app.insert_resource(test_registry());
        app.init_resource::<GameConfigValue>();
        app.init_resource::<WorldContext>();
        app.init_resource::<StreamingTimer>();
        app.add_message::<ChunkStreamError>();
        app
    }

    fn run_streaming_step(app: &mut App) {
        {
            let mut time = app.world_mut().resource_mut::<Time>();
            time.advance_by(std::time::Duration::from_secs_f32(STREAM_INTERVAL_SECS + 0.01));
        }
        stream_chunks(app.world_mut());
    }

    #[test]
    fn player_ships_are_never_offloaded() {
        let mut app = build_test_app();
        let player1 = app.world_mut().spawn((
            Ship { id: Uuid::now_v7(), ..default() },
            Team::new(0),
            Transform::from_xyz(0.0, 0.0, 0.0),
            BlockGrid::new(),
        )).id();
        let player2 = app.world_mut().spawn((
            Ship { id: Uuid::now_v7(), ..default() },
            Team::new(0),
            Transform::from_xyz(30_000.0, 0.0, 0.0),
            BlockGrid::new(),
        )).id();

        run_streaming_step(&mut app);

        assert!(app.world().get_entity(player1).is_ok());
        assert!(app.world().get_entity(player2).is_ok());
    }

    #[test]
    fn enemy_beyond_despawn_radius_is_offloaded_to_chunk_store() {
        let mut app = build_test_app();
        app.world_mut().spawn((
            Ship { id: Uuid::now_v7(), ..default() },
            Team::new(0),
            Transform::from_xyz(0.0, 0.0, 0.0),
            BlockGrid::new(),
        ));

        let enemy_id = Uuid::now_v7();
        let enemy = app.world_mut().spawn((
            Ship { id: enemy_id, ..default() },
            Team::new(1),
            Transform::from_xyz(15_000.0, 0.0, 0.0),
            BlockGrid::new(),
        )).id();

        run_streaming_step(&mut app);

        assert!(app.world().get_entity(enemy).is_err(), "enemy must be despawned from ECS");

        let context = app.world().resource::<WorldContext>();
        let store = context.store();
        let coord = ChunkCoord::from_world(Vec2::new(15_000.0, 0.0), CHUNK_SIZE);
        let chunk_data = store.load_chunk(coord).expect("load chunk").expect("chunk saved");
        assert_eq!(chunk_data.enemies.len(), 1);
        assert_eq!(chunk_data.enemies[0].id, enemy_id);
    }

    #[test]
    fn hysteresis_deadband_preserves_live_and_stored_states() {
        let mut app = build_test_app();
        app.world_mut().spawn((
            Ship { id: Uuid::now_v7(), ..default() },
            Team::new(0),
            Transform::from_xyz(0.0, 0.0, 0.0),
            BlockGrid::new(),
        ));

        let live_enemy = app.world_mut().spawn((
            Ship { id: Uuid::now_v7(), ..default() },
            Team::new(1),
            Transform::from_xyz(9_000.0, 0.0, 0.0),
            BlockGrid::new(),
        )).id();

        let stored_enemy_id = Uuid::now_v7();
        let coord = ChunkCoord::new(0, 0);
        let stored_enemy = ShipSnapshot {
            id: stored_enemy_id,
            team: 1,
            transform: TransformSave { x: 9_500.0, y: 0.0, rotation: 0.0 },
            velocity: VelocitySave { x: 0.0, y: 0.0, angvel: 0.0 },
            cargo: default(),
            doctrine: default(),
            skill: 5,
            brain: default(),
            blocks: Vec::new(),
        };
        {
            let context = app.world().resource::<WorldContext>();
            let store = context.store();
            store.save_chunk(coord, &ChunkData {
                enemies: vec![stored_enemy],
                debris: Vec::new(),
            }).expect("save chunk");
        }

        run_streaming_step(&mut app);

        assert!(app.world().get_entity(live_enemy).is_ok());

        let context = app.world().resource::<WorldContext>();
        let store = context.store();
        let chunk_data = store.load_chunk(coord).expect("load").expect("present");
        assert_eq!(chunk_data.enemies.len(), 1);
        assert_eq!(chunk_data.enemies[0].id, stored_enemy_id);
    }

    #[test]
    fn debris_offloads_and_preserves_remaining_lifetime() {
        let mut app = build_test_app();
        app.world_mut().spawn((
            Ship { id: Uuid::now_v7(), ..default() },
            Team::new(0),
            Transform::from_xyz(0.0, 0.0, 0.0),
            BlockGrid::new(),
        ));

        let debris = app.world_mut().spawn((
            Debris,
            Transform::from_xyz(12_000.0, 0.0, 0.0),
            Lifetime::seconds(45.5),
            BlockGrid::new(),
        )).id();

        run_streaming_step(&mut app);

        assert!(app.world().get_entity(debris).is_err());

        let context = app.world().resource::<WorldContext>();
        let store = context.store();
        let coord = ChunkCoord::from_world(Vec2::new(12_000.0, 0.0), CHUNK_SIZE);
        let chunk_data = store.load_chunk(coord).expect("load").expect("present");
        assert_eq!(chunk_data.debris.len(), 1);
        assert!((chunk_data.debris[0].remaining_secs - 45.5).abs() < 0.1);
    }

    #[test]
    fn stored_enemy_reloads_across_50k_chunk_border() {
        let mut app = build_test_app();
        app.world_mut().spawn((
            Ship { id: Uuid::now_v7(), ..default() },
            Team::new(0),
            Transform::from_xyz(49_000.0, 0.0, 0.0),
            BlockGrid::new(),
        ));

        let enemy_id = Uuid::now_v7();
        let neighbor_coord = ChunkCoord::new(1, 0);
        let stored_enemy = ShipSnapshot {
            id: enemy_id,
            team: 1,
            transform: TransformSave { x: 52_000.0, y: 0.0, rotation: 0.0 },
            velocity: VelocitySave { x: 0.0, y: 0.0, angvel: 0.0 },
            cargo: default(),
            doctrine: default(),
            skill: 5,
            brain: default(),
            blocks: vec![BlockSnapshot {
                placement: BlockPlacement::new(IVec2::ZERO, "core", Rot90::R0),
                health: 500.0,
                max_health: 500.0,
                battery_charge: None,
                turret: None,
                ammo: None,
                pattern: None,
            }],
        };
        {
            let context = app.world().resource::<WorldContext>();
            let store = context.store();
            store.save_chunk(neighbor_coord, &ChunkData {
                enemies: vec![stored_enemy],
                debris: Vec::new(),
            }).expect("save chunk");
        }

        run_streaming_step(&mut app);

        let context = app.world().resource::<WorldContext>();
        let store = context.store();
        assert_eq!(store.load_chunk(neighbor_coord).expect("load"), None);

        let mut query = app.world_mut().query::<&Ship>();
        let spawned: Vec<_> = query.iter(app.world()).filter(|s| s.id == enemy_id).collect();
        assert_eq!(spawned.len(), 1);
    }
}
