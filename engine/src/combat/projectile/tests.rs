use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::rapier::geometry::CollisionEventFlags;
use crate::common::{DamageKind, EnemyKind, Obstacle};
use crate::enemies::Enemy;
use crate::message::messages::DamageMessage;
use crate::config::weapons::{PierceConfig, WeaponDefinition, WeaponRoster};
use crate::config::ConfigHotReloadedMessage;

use super::state::{HitLog, Projectile};
use super::system::handle_projectile_collisions;

fn setup_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<CollisionEvent>()
        .add_message::<DamageMessage>()
        .add_systems(Update, handle_projectile_collisions);
    app
}

fn collision_event(e1: Entity, e2: Entity) -> CollisionEvent {
    CollisionEvent::Started(e1, e2, CollisionEventFlags::SENSOR)
}

fn spawn_test_projectile(
    app: &mut App,
    pos: Vec3,
    dir: Vec3,
    damage: f32,
    power: f32,
    penalty: f32,
    knockback: f32,
) -> Entity {
    let owner = app.world_mut().spawn_empty().id();
    app.world_mut()
        .spawn((
            Projectile {
                owner,
                damage,
                initial_damage: damage,
                remaining_power: power,
                penalty,
                knockback,
                direction: dir,
                radius: 0.1,
                lifetime: 2.0,
                kind: DamageKind::Ballistic,
                hit_log: HitLog::new(),
            },
            Transform::from_translation(pos),
        ))
        .id()
}

fn spawn_test_enemy(app: &mut App, pos: Vec3, kind: EnemyKind, pierce_cost: f32) -> Entity {
    app.world_mut()
        .spawn((
            Enemy {
                kind,
                move_speed: 4.0,
                radius: 0.5,
                half_height: 0.5,
                mass: 1.0,
                scrap_value: 1,
                pierce_cost,
                knockback_vel: Vec3::ZERO,
            },
            Transform::from_translation(pos),
        ))
        .id()
}

fn spawn_test_obstacle(app: &mut App, pos: Vec3) -> Entity {
    app.world_mut()
        .spawn((Obstacle, Transform::from_translation(pos)))
        .id()
}

fn damage_messages(app: &App) -> Vec<DamageMessage> {
    let msgs = app.world().resource::<Messages<DamageMessage>>();
    let mut cursor = msgs.get_cursor();
    cursor.read(msgs).cloned().collect()
}

#[test]
fn same_frame_events_resolve_walker_before_tank_regardless_of_event_order() {
    let mut app = setup_test_app();

    // Projectile at (0, 0, 0) flying along +Z
    let proj = spawn_test_projectile(
        &mut app,
        Vec3::ZERO,
        Vec3::Z,
        24.0, // initial damage
        1.6,  // power
        5.0,  // penalty
        3.5,  // knockback
    );

    // Walker at Z = 2.0 (cost 1.0)
    let walker = spawn_test_enemy(&mut app, Vec3::new(0.0, 0.0, 2.0), EnemyKind::Walker, 1.0);
    // Tank at Z = 5.0 (cost 4.0)
    let tank = spawn_test_enemy(&mut app, Vec3::new(0.0, 0.0, 5.0), EnemyKind::Tank, 4.0);

    // Write Tank collision event FIRST, then Walker collision event SECOND
    app.world_mut().write_message(collision_event(proj, tank));
    app.world_mut().write_message(collision_event(proj, walker));

    app.update();

    let msgs = damage_messages(&app);
    assert_eq!(msgs.len(), 2, "both Walker and Tank must take damage");

    // Walker must resolve first despite event arrival order
    assert_eq!(msgs[0].target, walker);
    assert_eq!(msgs[0].amount, 24.0);

    // Tank resolves second with drained damage: 24.0 - (1.0 * 5.0) = 19.0
    assert_eq!(msgs[1].target, tank);
    assert_eq!(msgs[1].amount, 19.0);

    // Projectile should be despawned because remaining power after Tank (1.6 - 1.0 - 4.0 = -3.4) <= 0
    assert!(app.world().get::<Projectile>(proj).is_none());
}

#[test]
fn nearer_obstacle_precedence_stops_shot_before_farther_enemy() {
    let mut app = setup_test_app();

    let proj = spawn_test_projectile(&mut app, Vec3::ZERO, Vec3::Z, 24.0, 2.0, 5.0, 3.5);
    // Obstacle at Z = 1.0
    let wall = spawn_test_obstacle(&mut app, Vec3::new(0.0, 0.0, 1.0));
    // Enemy behind obstacle at Z = 3.0
    let enemy = spawn_test_enemy(&mut app, Vec3::new(0.0, 0.0, 3.0), EnemyKind::Walker, 1.0);

    // Both events register in the same frame
    app.world_mut().write_message(collision_event(proj, enemy));
    app.world_mut().write_message(collision_event(proj, wall));

    app.update();

    let msgs = damage_messages(&app);
    assert!(
        msgs.is_empty(),
        "nearer obstacle must stop shot before enemy takes damage"
    );
    assert!(app.world().get::<Projectile>(proj).is_none());
}

#[test]
fn nearer_enemy_takes_damage_before_farther_obstacle_stops_shot() {
    let mut app = setup_test_app();

    let proj = spawn_test_projectile(&mut app, Vec3::ZERO, Vec3::Z, 24.0, 2.0, 5.0, 3.5);
    let enemy = spawn_test_enemy(&mut app, Vec3::new(0.0, 0.0, 1.0), EnemyKind::Walker, 1.0);
    let wall = spawn_test_obstacle(&mut app, Vec3::new(0.0, 0.0, 3.0));

    app.world_mut().write_message(collision_event(proj, wall));
    app.world_mut().write_message(collision_event(proj, enemy));

    app.update();

    let msgs = damage_messages(&app);
    assert_eq!(msgs.len(), 1, "nearer enemy takes damage");
    assert_eq!(msgs[0].target, enemy);
    assert!(app.world().get::<Projectile>(proj).is_none());
}

#[test]
fn tank_as_first_contact_exhausts_rifle_shot_immediately() {
    let mut app = setup_test_app();

    // Rifle shot with power 1.6, penalty 5.0, damage 24.0
    let proj = spawn_test_projectile(&mut app, Vec3::ZERO, Vec3::Z, 24.0, 1.6, 5.0, 3.5);
    // Tank with pierce_cost 4.0
    let tank = spawn_test_enemy(&mut app, Vec3::new(0.0, 0.0, 2.0), EnemyKind::Tank, 4.0);
    // Walker behind Tank at Z = 4.0
    let walker = spawn_test_enemy(&mut app, Vec3::new(0.0, 0.0, 4.0), EnemyKind::Walker, 1.0);

    app.world_mut().write_message(collision_event(proj, tank));
    app.world_mut().write_message(collision_event(proj, walker));

    app.update();

    let msgs = damage_messages(&app);
    assert_eq!(msgs.len(), 1, "only Tank is hit");
    assert_eq!(msgs[0].target, tank);
    assert_eq!(msgs[0].amount, 24.0);
    assert!(app.world().get::<Projectile>(proj).is_none());
}

#[test]
fn runner_costs_less_than_walker() {
    let mut app = setup_test_app();

    // Shot crosses Runner (cost 0.6) then Walker (cost 1.0)
    let proj = spawn_test_projectile(&mut app, Vec3::ZERO, Vec3::Z, 20.0, 2.0, 5.0, 2.0);
    let runner = spawn_test_enemy(&mut app, Vec3::new(0.0, 0.0, 1.0), EnemyKind::Runner, 0.6);
    let walker = spawn_test_enemy(&mut app, Vec3::new(0.0, 0.0, 2.0), EnemyKind::Walker, 1.0);

    app.world_mut().write_message(collision_event(proj, runner));
    app.world_mut().write_message(collision_event(proj, walker));

    app.update();

    let msgs = damage_messages(&app);
    assert_eq!(msgs.len(), 2);
    // Runner takes initial 20.0 damage, drains 0.6 * 5.0 = 3.0 damage
    assert_eq!(msgs[0].target, runner);
    assert_eq!(msgs[0].amount, 20.0);
    // Walker takes 20.0 - 3.0 = 17.0 damage
    assert_eq!(msgs[1].target, walker);
    assert_eq!(msgs[1].amount, 17.0);
}

#[test]
fn zero_power_preserves_one_hit_legacy_behaviour() {
    let mut app = setup_test_app();

    // Legacy weapon with 0 power and 0 penalty
    let proj = spawn_test_projectile(&mut app, Vec3::ZERO, Vec3::Z, 30.0, 0.0, 0.0, 4.0);
    let e1 = spawn_test_enemy(&mut app, Vec3::new(0.0, 0.0, 1.0), EnemyKind::Walker, 1.0);
    let e2 = spawn_test_enemy(&mut app, Vec3::new(0.0, 0.0, 2.0), EnemyKind::Walker, 1.0);

    app.world_mut().write_message(collision_event(proj, e1));
    app.world_mut().write_message(collision_event(proj, e2));

    app.update();

    let msgs = damage_messages(&app);
    assert_eq!(msgs.len(), 1, "only the first body takes damage");
    assert_eq!(msgs[0].target, e1);
    assert_eq!(msgs[0].amount, 30.0);
    assert!(app.world().get::<Projectile>(proj).is_none());
}

#[test]
fn knockback_decays_with_retained_damage_fraction() {
    let mut app = setup_test_app();

    // Damage 100.0, knockback 10.0, power 2.0, penalty 50.0
    let proj = spawn_test_projectile(&mut app, Vec3::ZERO, Vec3::Z, 100.0, 2.0, 50.0, 10.0);
    let e1 = spawn_test_enemy(&mut app, Vec3::new(0.0, 0.0, 1.0), EnemyKind::Walker, 1.0);
    let e2 = spawn_test_enemy(&mut app, Vec3::new(0.0, 0.0, 2.0), EnemyKind::Walker, 1.0);

    app.world_mut().write_message(collision_event(proj, e1));
    app.world_mut().write_message(collision_event(proj, e2));

    app.update();

    let msgs = damage_messages(&app);
    assert_eq!(msgs.len(), 2);
    // First enemy: full knockback (100/100 * 10 = 10 along +Z)
    assert_eq!(msgs[0].target, e1);
    assert_eq!(msgs[0].knockback, Vec3::new(0.0, 0.0, 10.0));

    // Second enemy: damage was drained by 1.0 * 50 = 50.0, retained fraction is 50/100 = 0.5
    // knockback is 0.5 * 10 = 5.0 along +Z
    assert_eq!(msgs[1].target, e2);
    assert_eq!(msgs[1].amount, 50.0);
    assert_eq!(msgs[1].knockback, Vec3::new(0.0, 0.0, 5.0));
}

#[test]
fn hit_once_guarantee_ignores_repeated_event_for_same_enemy() {
    let mut app = setup_test_app();

    let proj = spawn_test_projectile(&mut app, Vec3::ZERO, Vec3::Z, 50.0, 2.0, 5.0, 5.0);
    let enemy = spawn_test_enemy(&mut app, Vec3::new(0.0, 0.0, 1.0), EnemyKind::Walker, 1.0);

    // Event 1 in frame 1
    app.world_mut().write_message(collision_event(proj, enemy));
    app.update();

    let msgs = damage_messages(&app);
    assert_eq!(msgs.len(), 1);

    // Repeated event for the same enemy in frame 2
    app.world_mut().write_message(collision_event(proj, enemy));
    app.update();

    let msgs_after = damage_messages(&app);
    assert_eq!(msgs_after.len(), 1, "repeated event must be ignored");
}

#[test]
fn weapon_roster_hot_reload_applies_only_to_newly_spawned_projectiles() {
    use crate::combat::weapon::{sync_weapons_hot_reload, WeaponInventory};
    use crate::combat::projectile::spawn_projectile;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<ConfigHotReloadedMessage>()
        .add_systems(Update, sync_weapons_hot_reload);

    let mut def = WeaponDefinition {
        id: "test_gun".to_string(),
        pierce: PierceConfig {
            power: 1.0,
            penalty: 2.0,
        },
        projectile_speed: 40.0,
        projectile_radius: 0.1,
        projectile_lifetime: 1.0,
        knockback: 3.0,
        ..Default::default()
    };

    let roster = WeaponRoster {
        weapons: vec![def.clone()],
        ..Default::default()
    };
    app.insert_resource(roster.clone());

    // Spawn survivor with inventory
    let shooter = app.world_mut().spawn(WeaponInventory::from_config(&roster)).id();

    // Spawn projectile BEFORE hot reload
    let mut commands = app.world_mut().commands();
    let old_proj_entity = spawn_projectile(
        &mut commands,
        shooter,
        Vec3::ZERO,
        Vec3::Z,
        25.0,
        &def,
    );
    app.update();

    let old_proj = app.world().get::<Projectile>(old_proj_entity).expect("old proj exists");
    assert_eq!(old_proj.remaining_power, 1.0);
    assert_eq!(old_proj.penalty, 2.0);

    // Trigger hot reload with updated pierce config
    def.pierce.power = 3.5;
    def.pierce.penalty = 7.0;
    app.world_mut().resource_mut::<WeaponRoster>().weapons = vec![def.clone()];
    app.world_mut().write_message(ConfigHotReloadedMessage);
    app.update();

    // In-flight projectile retains old profile
    let old_proj_after = app.world().get::<Projectile>(old_proj_entity).expect("old proj exists");
    assert_eq!(old_proj_after.remaining_power, 1.0);
    assert_eq!(old_proj_after.penalty, 2.0);

    // Newly spawned projectile picks up the new profile
    let active_weapon = {
        let inv = app.world().get::<WeaponInventory>(shooter).expect("inventory exists");
        let active_slot = inv.active().expect("active slot");
        assert_eq!(active_slot.weapon.pierce.power, 3.5);
        active_slot.weapon.clone()
    };

    let mut commands = app.world_mut().commands();
    let new_proj_entity = spawn_projectile(
        &mut commands,
        shooter,
        Vec3::ZERO,
        Vec3::Z,
        25.0,
        &active_weapon,
    );
    app.update();

    let new_proj = app.world().get::<Projectile>(new_proj_entity).expect("new proj exists");
    assert_eq!(new_proj.remaining_power, 3.5);
    assert_eq!(new_proj.penalty, 7.0);
}

#[test]
fn sync_enemies_hot_reload_updates_live_enemy_pierce_cost() {
    use crate::enemies::ai::system::sync_enemies_hot_reload;
    use crate::config::EnemiesConfig;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<ConfigHotReloadedMessage>()
        .add_systems(Update, sync_enemies_hot_reload);

    let enemies_cfg: EnemiesConfig = crate::assert_ron_config_parses!(EnemiesConfig);
    app.insert_resource(enemies_cfg);

    let enemy_entity = spawn_test_enemy(&mut app, Vec3::ZERO, EnemyKind::Walker, 1.0);
    app.update();

    let initial = app.world().get::<Enemy>(enemy_entity).expect("enemy exists");
    assert_eq!(initial.pierce_cost, 1.0);

    // Modify config and hot reload
    app.world_mut().resource_mut::<EnemiesConfig>().archetypes[0].pierce_cost = 3.25;
    app.world_mut().write_message(ConfigHotReloadedMessage);
    app.update();

    let after = app.world().get::<Enemy>(enemy_entity).expect("enemy exists");
    assert_eq!(after.pierce_cost, 3.25);
}
