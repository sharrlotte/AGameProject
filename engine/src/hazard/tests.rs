use std::time::Duration;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use crate::common::DamageKind;
use crate::config::HazardConfig;
use crate::message::messages::DamageMessage;
use crate::world::Survivor;

use super::state::{HazardSpec, HazardSpawnMessage, HazardZone};
use super::system::{spawn_hazards, tick_hazards};

#[derive(Resource, Default)]
struct Recorded(Vec<DamageMessage>);

fn record(mut reader: MessageReader<DamageMessage>, mut recorded: ResMut<Recorded>) {
    recorded.0.extend(reader.read().copied());
}

fn test_app(tick_interval: f32, max_active: usize) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(0.1)))
        .insert_resource(HazardConfig { max_active, tick_interval })
        .add_message::<HazardSpawnMessage>()
        .add_message::<DamageMessage>()
        .init_resource::<Recorded>()
        .add_systems(Update, (tick_hazards, spawn_hazards, record).chain());
    app
}

fn spec(radius: f32, damage: f32, lifetime: f32) -> HazardSpec {
    HazardSpec {
        radius,
        damage,
        lifetime,
        kind: DamageKind::Acid,
    }
}

fn request(app: &mut App, position: Vec3, spec: HazardSpec) {
    app.world_mut()
        .write_message(HazardSpawnMessage { position, spec });
}

fn spawn_survivor(app: &mut App, at: Vec3) -> Entity {
    app.world_mut()
        .spawn((Survivor, Transform::from_translation(at)))
        .id()
}

fn zones(app: &mut App) -> Vec<(Vec3, HazardZone)> {
    let world = app.world_mut();
    world
        .query::<(&Transform, &HazardZone)>()
        .iter(world)
        .map(|(transform, zone)| (transform.translation, *zone))
        .collect()
}

#[test]
fn zone_spawns_from_spec() {
    let mut app = test_app(0.5, 12);
    request(&mut app, Vec3::new(3.0, 4.0, -2.0), spec(2.5, 8.0, 6.0));
    app.update();

    let zones = zones(&mut app);
    assert_eq!(zones.len(), 1);
    let (position, zone) = zones[0];
    assert!((position.x - 3.0).abs() < 1e-6);
    assert!((position.z + 2.0).abs() < 1e-6);
    assert_eq!(position.y, 0.0, "zones live at ground height");
    assert!((zone.spec.radius - 2.5).abs() < 1e-6);
    assert!((zone.spec.damage - 8.0).abs() < 1e-6);
    assert!((zone.spec.lifetime - 6.0).abs() < 1e-6);
    assert_eq!(zone.spec.kind, DamageKind::Acid);
    assert!((zone.tick_timer - 0.5).abs() < 1e-6);
}

#[test]
fn first_tick_waits_one_interval_then_repeats() {
    let mut app = test_app(0.5, 12);
    let survivor = spawn_survivor(&mut app, Vec3::ZERO);
    request(&mut app, Vec3::ZERO, spec(2.5, 8.0, 6.0));
    app.update();
    assert_eq!(app.world().resource::<Recorded>().0.len(), 0);

    for _ in 0..4 {
        app.update();
    }
    assert_eq!(app.world().resource::<Recorded>().0.len(), 0);

    for _ in 0..2 {
        app.update();
    }
    let recorded = &app.world().resource::<Recorded>().0;
    assert_eq!(recorded.len(), 1);
    assert_eq!(recorded[0].target, survivor);
    assert!((recorded[0].amount - 8.0).abs() < 1e-6);
    assert_eq!(recorded[0].kind, DamageKind::Acid);

    for _ in 0..10 {
        app.update();
    }
    let recorded = &app.world().resource::<Recorded>().0;
    assert!(recorded.len() >= 2);
    assert!(recorded.iter().all(|msg| (msg.amount - 8.0).abs() < 1e-6));
}

#[test]
fn targets_outside_the_radius_are_ignored() {
    let mut app = test_app(0.5, 12);
    spawn_survivor(&mut app, Vec3::new(10.0, 0.9, 0.0));
    request(&mut app, Vec3::ZERO, spec(2.5, 8.0, 6.0));

    for _ in 0..12 {
        app.update();
    }
    assert!(app.world().resource::<Recorded>().0.is_empty());
}

#[test]
fn pools_do_not_damage_enemies() {
    let mut app = test_app(0.5, 12);
    app.world_mut()
        .spawn((Name::new("Enemy"), Transform::from_translation(Vec3::ZERO)));
    request(&mut app, Vec3::ZERO, spec(2.5, 8.0, 6.0));

    for _ in 0..12 {
        app.update();
    }
    assert!(app.world().resource::<Recorded>().0.is_empty());
}

#[test]
fn zones_expire_after_their_lifetime() {
    let mut app = test_app(0.5, 12);
    spawn_survivor(&mut app, Vec3::ZERO);
    request(&mut app, Vec3::ZERO, spec(2.5, 8.0, 0.3));

    app.update();
    assert_eq!(zones(&mut app).len(), 1);

    for _ in 0..4 {
        app.update();
    }
    assert!(zones(&mut app).is_empty(), "zone must despawn after its lifetime");
}

#[test]
fn cap_retires_the_oldest_zone() {
    let mut app = test_app(0.5, 2);
    request(&mut app, Vec3::new(1.0, 0.0, 0.0), spec(2.5, 8.0, 10.0));
    app.update();

    for _ in 0..3 {
        app.update();
    }
    request(&mut app, Vec3::new(2.0, 0.0, 0.0), spec(2.5, 8.0, 10.0));
    app.update();
    request(&mut app, Vec3::new(3.0, 0.0, 0.0), spec(2.5, 8.0, 10.0));
    app.update();

    let positions: Vec<f32> = zones(&mut app).iter().map(|(pos, _)| pos.x).collect();
    assert_eq!(positions.len(), 2);
    assert!(!positions.contains(&1.0), "oldest zone must be retired");
    assert!(positions.contains(&2.0));
    assert!(positions.contains(&3.0));
}
