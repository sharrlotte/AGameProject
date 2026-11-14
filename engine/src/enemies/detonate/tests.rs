use std::time::Duration;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;

use crate::common::{Dying, EnemyKind};
use crate::message::messages::{DamageMessage, DetonationMessage, EnemyKilledMessage};
use crate::world::{Health, Survivor};

use crate::enemies::ai::state::Enemy;
use crate::enemies::brain::{BrainState, EnemyBrain};

use super::state::{Detonated, Fuse};
use super::system::{blast_falloff, resolve_detonations, update_fuses};

#[derive(Resource, Default)]
struct Recorded {
    damage: Vec<DamageMessage>,
    kills: Vec<EnemyKilledMessage>,
    blasts: Vec<DetonationMessage>,
}

fn record(
    mut damage: MessageReader<DamageMessage>,
    mut kills: MessageReader<EnemyKilledMessage>,
    mut blasts: MessageReader<DetonationMessage>,
    mut recorded: ResMut<Recorded>,
) {
    recorded.damage.extend(damage.read().copied());
    recorded.kills.extend(kills.read().copied());
    recorded.blasts.extend(blasts.read().copied());
}

fn blast_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(0.1)))
        .add_message::<DamageMessage>()
        .add_message::<EnemyKilledMessage>()
        .add_message::<DetonationMessage>()
        .init_resource::<Recorded>()
        .add_systems(Update, (update_fuses, resolve_detonations, record).chain());
    app
}

fn archetype(kind: EnemyKind) -> crate::config::EnemyArchetypeData {
    let config: crate::config::EnemiesConfig = crate::assert_ron_config_parses!(crate::config::EnemiesConfig);
    config.archetype(kind).expect("authored archetype").clone()
}

fn spawn_survivor(app: &mut App, at: Vec3) -> Entity {
    app.world_mut()
        .spawn((Survivor, Transform::from_translation(at)))
        .id()
}

fn spawn_exploder(app: &mut App, at: Vec3, target: Option<Entity>, armed: bool) -> Entity {
    let arch = archetype(EnemyKind::Exploder);
    let spec = arch.detonate().expect("exploder detonate");
    let mut fuse = Fuse::from_spec(spec);
    fuse.armed = armed;
    fuse.timer = 0.0;
    app.world_mut()
        .spawn((
            Enemy::from_archetype(&arch),
            EnemyBrain {
                state: BrainState::Engage,
                target,
                last_known: Some(Vec3::ZERO),
                ..default()
            },
            fuse,
            Health::new(arch.max_hp),
            Transform::from_translation(at),
        ))
        .id()
}

fn spawn_walker(app: &mut App, at: Vec3, dying: bool) -> Entity {
    let arch = archetype(EnemyKind::Walker);
    let mut entity = app.world_mut().spawn((
        Enemy::from_archetype(&arch),
        Health::new(arch.max_hp),
        Transform::from_translation(at),
    ));
    if dying {
        entity.insert(Dying);
    }
    entity.id()
}

#[test]
fn proximity_arms_an_irreversible_fuse() {
    let mut app = blast_app();
    let survivor = spawn_survivor(&mut app, Vec3::ZERO);
    let exploder = spawn_exploder(&mut app, Vec3::new(0.0, 0.9, 2.0), Some(survivor), false);

    app.update();
    let fuse = *app.world().get::<Fuse>(exploder).expect("fuse");
    assert!(fuse.armed, "survivor inside the trigger radius must arm the fuse");
    assert!(fuse.timer <= fuse.fuse);

    app.world_mut()
        .get_mut::<Transform>(survivor)
        .expect("survivor transform")
        .translation = Vec3::new(0.0, 0.9, 100.0);
    app.update();
    let fuse = *app.world().get::<Fuse>(exploder).expect("fuse");
    assert!(fuse.armed, "fuse is irreversible");
    assert!(fuse.timer < fuse.fuse, "fuse must count down after arming");

    for _ in 0..10 {
        app.update();
    }
    assert!(app.world().get::<Detonated>(exploder).is_some(), "fuse must expire");
    assert_eq!(app.world().resource::<Recorded>().kills.len(), 1);
    assert_eq!(app.world().resource::<Recorded>().kills[0].scrap, 4);
}

#[test]
fn blast_damages_survivor_and_living_enemies_only() {
    let mut app = blast_app();
    let survivor = spawn_survivor(&mut app, Vec3::new(0.0, 0.9, 1.0));
    let walker = spawn_walker(&mut app, Vec3::new(0.0, 0.9, -1.0), false);
    let dying = spawn_walker(&mut app, Vec3::new(0.5, 0.9, 0.0), true);
    let exploder = spawn_exploder(&mut app, Vec3::ZERO, Some(survivor), true);

    app.update();

    let recorded = app.world().resource::<Recorded>();
    assert_eq!(recorded.blasts.len(), 1);
    assert_eq!(recorded.damage.len(), 2, "survivor and one living enemy");

    let survivor_hit = recorded
        .damage
        .iter()
        .find(|msg| msg.target == survivor)
        .expect("survivor hit");
    let walker_hit = recorded
        .damage
        .iter()
        .find(|msg| msg.target == walker)
        .expect("walker hit");
    assert!(survivor_hit.amount > 0.0 && survivor_hit.amount < 45.0);
    assert!((survivor_hit.amount - walker_hit.amount).abs() < 1e-4);
    assert!(survivor_hit.knockback.z > 0.0, "knockback points away from the blast");
    assert!(walker_hit.knockback.z < 0.0, "knockback points away from the blast");
    assert!(!recorded.damage.iter().any(|msg| msg.target == dying));
    assert!(!recorded.damage.iter().any(|msg| msg.target == exploder));
}

#[test]
fn dying_exploder_detonates_once_without_reporting_a_kill() {
    let mut app = blast_app();
    let exploder = spawn_exploder(&mut app, Vec3::ZERO, None, false);
    app.world_mut().entity_mut(exploder).insert(Dying);
    app.world_mut()
        .get_mut::<Health>(exploder)
        .expect("health")
        .current = 0.0;

    app.update();
    assert_eq!(app.world().resource::<Recorded>().blasts.len(), 1);
    assert!(app.world().resource::<Recorded>().kills.is_empty());

    app.update();
    assert_eq!(app.world().resource::<Recorded>().blasts.len(), 1, "one blast per entity");
}

#[test]
fn blast_falloff_reaches_zero_at_the_radius() {
    assert!((blast_falloff(0.0, 4.5) - 1.0).abs() < 1e-6);
    assert!((blast_falloff(2.25, 4.5) - 0.5).abs() < 1e-6);
    assert_eq!(blast_falloff(4.5, 4.5), 0.0);
    assert_eq!(blast_falloff(10.0, 4.5), 0.0);
    assert_eq!(blast_falloff(f32::NAN, 4.5), 0.0);
}
