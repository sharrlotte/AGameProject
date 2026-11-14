use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::rapier::geometry::CollisionEventFlags;

use crate::common::{GROUP_WORLD, Obstacle};
use crate::world::Survivor;
use super::loader::{
    apply_roof_cutaways, detect_survivor_interiors, inspect_map_scene_nodes,
    reposition_player_to_spawn_marker, MapNodeProcessed,
};
use super::map::{
    ActiveInteriors, ColliderMesh, EnemySpawnerMarker, InteriorVolume, PlayerSpawnMarker,
    RoofOcclusion,
};

#[test]
fn test_inspect_map_scene_nodes_classification() {
    let mut app = App::new();
    app.init_resource::<Assets<Mesh>>();
    app.add_systems(Update, inspect_map_scene_nodes);

    let col_entity = app
        .world_mut()
        .spawn((Name::new("Collider_Cover_01"), Transform::default()))
        .id();
    let player_spawn = app
        .world_mut()
        .spawn((Name::new("Spawn_Player"), Transform::default()))
        .id();
    let enemy_spawn_default = app
        .world_mut()
        .spawn((Name::new("Spawn_Enemy"), Transform::default()))
        .id();
    let enemy_spawn_typed = app
        .world_mut()
        .spawn((Name::new("Spawn_Enemy_Brute"), Transform::default()))
        .id();
    let interior_trigger = app
        .world_mut()
        .spawn((Name::new("Trigger_Interior_gas_station"), Transform::default()))
        .id();
    let roof_node = app
        .world_mut()
        .spawn((Name::new("Roof_gas_station"), Transform::default()))
        .id();
    let wall_node = app
        .world_mut()
        .spawn((Name::new("UpperWall_gas_station"), Transform::default()))
        .id();
    let scenery_node = app
        .world_mut()
        .spawn((Name::new("Ground_Tile"), Transform::default()))
        .id();

    app.update();

    let world = app.world();

    // Collider
    assert!(world.get::<ColliderMesh>(col_entity).is_some());
    assert!(world.get::<Obstacle>(col_entity).is_some());
    assert!(world.get::<RigidBody>(col_entity).is_some());
    assert_eq!(
        world.get::<Visibility>(col_entity),
        Some(&Visibility::Hidden)
    );
    let groups = world
        .get::<CollisionGroups>(col_entity)
        .expect("collision groups");
    assert_eq!(groups.memberships, GROUP_WORLD);

    // Spawns
    assert!(world.get::<PlayerSpawnMarker>(player_spawn).is_some());

    let def_marker = world
        .get::<EnemySpawnerMarker>(enemy_spawn_default)
        .expect("enemy marker");
    assert_eq!(def_marker.kind_filter, None);

    let typed_marker = world
        .get::<EnemySpawnerMarker>(enemy_spawn_typed)
        .expect("typed enemy marker");
    assert_eq!(typed_marker.kind_filter.as_deref(), Some("Brute"));

    // Interior & Roofs
    let interior = world
        .get::<InteriorVolume>(interior_trigger)
        .expect("interior volume");
    assert_eq!(interior.id, "gas_station");
    assert!(world.get::<Sensor>(interior_trigger).is_some());

    let roof = world
        .get::<RoofOcclusion>(roof_node)
        .expect("roof occlusion");
    assert_eq!(roof.interior_id, "gas_station");

    let wall = world
        .get::<RoofOcclusion>(wall_node)
        .expect("wall occlusion");
    assert_eq!(wall.interior_id, "gas_station");

    // Scenery untouched
    assert!(world.get::<ColliderMesh>(scenery_node).is_none());
    assert!(world.get::<InteriorVolume>(scenery_node).is_none());
    assert!(world.get::<MapNodeProcessed>(scenery_node).is_some());
}

#[test]
fn test_detect_survivor_interiors_enter_and_exit() {
    let mut app = App::new();
    app.add_message::<CollisionEvent>();
    app.add_systems(Update, detect_survivor_interiors);

    let survivor = app
        .world_mut()
        .spawn((Survivor, ActiveInteriors::default()))
        .id();
    let interior = app
        .world_mut()
        .spawn(InteriorVolume {
            id: "gas_station".to_string(),
        })
        .id();

    // Started collision event
    app.world_mut().write_message(CollisionEvent::Started(
        survivor,
        interior,
        CollisionEventFlags::SENSOR,
    ));
    app.update();

    let active = app
        .world()
        .get::<ActiveInteriors>(survivor)
        .expect("active interiors");
    assert_eq!(active.ids, vec!["gas_station".to_string()]);

    // Stopped collision event
    app.world_mut().write_message(CollisionEvent::Stopped(
        survivor,
        interior,
        CollisionEventFlags::SENSOR,
    ));
    app.update();

    let active = app
        .world()
        .get::<ActiveInteriors>(survivor)
        .expect("active interiors");
    assert!(active.ids.is_empty());
}

#[test]
fn test_apply_roof_cutaways() {
    let mut app = App::new();
    app.add_systems(Update, apply_roof_cutaways);

    let survivor = app
        .world_mut()
        .spawn((
            Survivor,
            ActiveInteriors {
                ids: vec!["gas_station".to_string()],
            },
        ))
        .id();

    let roof_gas = app
        .world_mut()
        .spawn((
            RoofOcclusion {
                interior_id: "gas_station".to_string(),
            },
            Visibility::Inherited,
        ))
        .id();

    let roof_other = app
        .world_mut()
        .spawn((
            RoofOcclusion {
                interior_id: "diner".to_string(),
            },
            Visibility::Inherited,
        ))
        .id();

    app.update();

    assert_eq!(
        app.world().get::<Visibility>(roof_gas),
        Some(&Visibility::Hidden)
    );
    assert_eq!(
        app.world().get::<Visibility>(roof_other),
        Some(&Visibility::Inherited)
    );

    // Survivor exits
    app.world_mut()
        .get_mut::<ActiveInteriors>(survivor)
        .expect("active")
        .ids
        .clear();
    app.update();

    assert_eq!(
        app.world().get::<Visibility>(roof_gas),
        Some(&Visibility::Inherited)
    );
}

#[test]
fn test_reposition_player_to_spawn_marker() {
    let mut app = App::new();
    app.add_systems(Update, reposition_player_to_spawn_marker);

    let player = app
        .world_mut()
        .spawn((Survivor, Transform::from_xyz(0.0, 0.0, 0.0)))
        .id();

    app.world_mut().spawn((
        PlayerSpawnMarker,
        Transform::from_xyz(12.5, 0.0, -8.0),
    ));

    app.update();

    let player_tf = app
        .world()
        .get::<Transform>(player)
        .expect("player transform");
    assert_eq!(player_tf.translation, Vec3::new(12.5, 0.0, -8.0));
}
