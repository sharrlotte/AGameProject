use std::time::Duration;

use bevy::prelude::*;
use crate::config::ConfigHotReloadedMessage;
use crate::message::messages::{
    PlayerDiedMessage, ReloadCancelReason, ReloadCancelledMessage, ReloadFinishedMessage,
    ReloadStartedMessage, WeaponFiredMessage, WeaponSwappedMessage,
};
use crate::message::resources::UpgradeModifiers;
use crate::world::player::spawn_player;
use crate::world::{ActionIntent, Health, PlayerConfig, Stamina, Survivor};

use crate::config::WeaponRoster;
use super::projectile::Projectile;
use super::reload::{cancel_reload_on_death, tick_and_start_reload};
use super::stance::{CombatStance, update_combat_stance};
use super::weapon::{
    WeaponInventory, attach_weapon_inventory, handle_weapon_fire, handle_weapon_swap,
};

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(crate::assert_ron_config_parses!(WeaponRoster))
        .init_resource::<PlayerConfig>()
        .init_resource::<UpgradeModifiers>()
        .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            Duration::from_secs_f32(1.0 / 60.0),
        ))
        .add_message::<WeaponFiredMessage>()
        .add_message::<ReloadStartedMessage>()
        .add_message::<ReloadFinishedMessage>()
        .add_message::<ReloadCancelledMessage>()
        .add_message::<WeaponSwappedMessage>()
        .add_message::<ConfigHotReloadedMessage>()
        .add_message::<PlayerDiedMessage>()
        .add_systems(Startup, spawn_player)
        .add_systems(
            Update,
            (
                attach_weapon_inventory,
                handle_weapon_swap,
                tick_and_start_reload,
                handle_weapon_fire,
                update_combat_stance,
                cancel_reload_on_death,
            )
                .chain(),
        );
    app
}

fn messages<T: Message + Clone>(app: &App) -> Vec<T> {
    let msgs = app.world().resource::<Messages<T>>();
    let mut cursor = msgs.get_cursor();
    cursor.read(msgs).cloned().collect()
}

fn player_entity(app: &mut App) -> Entity {
    app.world_mut()
        .query_filtered::<Entity, With<Survivor>>()
        .single(app.world())
        .expect("player spawned")
}

fn step_sim(app: &mut App, secs: f32) {
    let steps = (secs / (1.0 / 60.0)).ceil().max(1.0) as usize;
    for _ in 0..steps {
        app.update();
    }
}

#[test]
fn rifle_fires_shotgun_swaps_and_reload_cancels_on_swap() {
    let mut app = test_app();
    app.update();
    let player = player_entity(&mut app);

    let roster = app.world().resource::<WeaponRoster>().clone();
    let rifle_mag = roster.weapons[0].mag_size;
    let shotgun_pellets = roster.weapons[1].pellets;

    app.world_mut()
        .get_mut::<ActionIntent>(player)
        .unwrap()
        .fire_held = true;
    step_sim(&mut app, 0.05);

    let proj_count = app.world_mut().query::<&Projectile>().iter(app.world()).count();
    assert_eq!(proj_count, 1);
    assert_eq!(
        app.world()
            .get::<WeaponInventory>(player)
            .unwrap()
            .active()
            .unwrap()
            .ammo,
        rifle_mag - 1
    );

    {
        let mut intent = app.world_mut().get_mut::<ActionIntent>(player).unwrap();
        intent.fire_held = false;
        intent.reload_just_pressed = true;
    }
    step_sim(&mut app, 0.05);
    assert!(
        app.world()
            .get::<WeaponInventory>(player)
            .unwrap()
            .reload_timer
            .is_some()
    );

    app.world_mut()
        .get_mut::<ActionIntent>(player)
        .unwrap()
        .swap_slot = Some(1);
    step_sim(&mut app, 0.20);
    {
        let inv = app.world().get::<WeaponInventory>(player).unwrap();
        assert_eq!(inv.active_slot, 1);
        assert!(inv.reload_timer.is_none());
    }

    app.world_mut()
        .get_mut::<ActionIntent>(player)
        .unwrap()
        .fire_just_pressed = true;
    step_sim(&mut app, 0.05);

    let total_projs = app.world_mut().query::<&Projectile>().iter(app.world()).count();
    assert_eq!(total_projs, 1 + shotgun_pellets as usize);
}

#[test]
fn empty_trigger_starts_auto_reload_and_refills_mag() {
    let mut app = test_app();
    app.update();
    let player = player_entity(&mut app);

    let roster = app.world().resource::<WeaponRoster>().clone();
    let rifle = &roster.weapons[0];

    app.world_mut()
        .get_mut::<WeaponInventory>(player)
        .unwrap()
        .active_mut()
        .unwrap()
        .ammo = 0;

    app.world_mut()
        .get_mut::<ActionIntent>(player)
        .unwrap()
        .fire_held = true;
    step_sim(&mut app, 0.05);

    assert!(
        app.world()
            .get::<WeaponInventory>(player)
            .unwrap()
            .reload_timer
            .is_some()
    );

    app.world_mut()
        .get_mut::<ActionIntent>(player)
        .unwrap()
        .fire_held = false;
    step_sim(&mut app, rifle.reload_secs + 0.1);

    let inv = app.world().get::<WeaponInventory>(player).unwrap();
    assert!(inv.reload_timer.is_none());
    assert_eq!(inv.active().unwrap().ammo, rifle.mag_size);
}

#[test]
fn stance_enters_from_aim_fire_and_reload_and_exits_after_hold() {
    let mut app = test_app();
    app.update();
    let player = player_entity(&mut app);
    let hold = app.world().resource::<WeaponRoster>().weapons[0].stance_hold_secs;

    app.world_mut()
        .get_mut::<ActionIntent>(player)
        .unwrap()
        .aim_held = true;
    app.update();
    assert!(app.world().get::<CombatStance>(player).unwrap().aiming);

    app.world_mut()
        .get_mut::<ActionIntent>(player)
        .unwrap()
        .aim_held = false;
    app.update();
    assert!(
        app.world().get::<CombatStance>(player).unwrap().aiming,
        "hold timer should keep the stance raised"
    );
    step_sim(&mut app, hold + 0.1);
    assert!(!app.world().get::<CombatStance>(player).unwrap().aiming);

    app.world_mut()
        .get_mut::<ActionIntent>(player)
        .unwrap()
        .fire_held = true;
    app.update();
    assert!(app.world().get::<CombatStance>(player).unwrap().aiming);
    app.world_mut()
        .get_mut::<ActionIntent>(player)
        .unwrap()
        .fire_held = false;
    step_sim(&mut app, hold + 0.1);
    assert!(!app.world().get::<CombatStance>(player).unwrap().aiming);

    app.world_mut()
        .get_mut::<WeaponInventory>(player)
        .unwrap()
        .active_mut()
        .unwrap()
        .ammo = 0;
    app.world_mut()
        .get_mut::<ActionIntent>(player)
        .unwrap()
        .reload_just_pressed = true;
    app.update();
    assert!(
        app.world()
            .get::<WeaponInventory>(player)
            .unwrap()
            .reload_timer
            .is_some()
    );
    assert!(app.world().get::<CombatStance>(player).unwrap().aiming);

    let reload_secs = app.world().resource::<WeaponRoster>().weapons[0].reload_secs;
    step_sim(&mut app, reload_secs + 0.1);
    assert!(
        app.world()
            .get::<WeaponInventory>(player)
            .unwrap()
            .reload_timer
            .is_none()
    );
    assert!(
        app.world().get::<CombatStance>(player).unwrap().aiming,
        "stance holds after the reload finishes"
    );
    step_sim(&mut app, hold + 0.1);
    assert!(!app.world().get::<CombatStance>(player).unwrap().aiming);
}

#[test]
fn dead_players_are_not_in_combat_stance() {
    let mut app = test_app();
    app.update();
    let player = player_entity(&mut app);

    app.world_mut()
        .get_mut::<ActionIntent>(player)
        .unwrap()
        .aim_held = true;
    app.update();
    assert!(app.world().get::<CombatStance>(player).unwrap().aiming);

    app.world_mut().get_mut::<Health>(player).unwrap().current = 0.0;
    app.update();
    assert!(!app.world().get::<CombatStance>(player).unwrap().aiming);
}

#[test]
fn swap_cancels_reload_without_partial_refill_and_emits_events() {
    let mut app = test_app();
    app.update();
    let player = player_entity(&mut app);

    app.world_mut()
        .get_mut::<WeaponInventory>(player)
        .unwrap()
        .active_mut()
        .unwrap()
        .ammo = 10;
    app.world_mut()
        .get_mut::<ActionIntent>(player)
        .unwrap()
        .reload_just_pressed = true;
    step_sim(&mut app, 0.2);
    let progress = app
        .world()
        .get::<WeaponInventory>(player)
        .unwrap()
        .reload_progress()
        .expect("reload active");
    assert!(progress > 0.0 && progress < 1.0);

    app.world_mut()
        .get_mut::<ActionIntent>(player)
        .unwrap()
        .swap_slot = Some(1);
    app.update();

    let inv = app.world().get::<WeaponInventory>(player).unwrap();
    assert_eq!(inv.active_slot, 1);
    assert!(inv.reload_timer.is_none());
    assert_eq!(inv.slots[0].ammo, 10, "cancelled reload must not refill");

    let cancelled = messages::<ReloadCancelledMessage>(&app);
    assert!(
        cancelled
            .iter()
            .any(|msg| msg.entity == player && msg.reason == ReloadCancelReason::Swapped)
    );
    let swapped = messages::<WeaponSwappedMessage>(&app);
    assert!(
        swapped
            .iter()
            .any(|msg| msg.entity == player && msg.from == 0 && msg.to == 1)
    );
}

#[test]
fn selecting_the_active_slot_keeps_the_reload_running() {
    let mut app = test_app();
    app.update();
    let player = player_entity(&mut app);

    app.world_mut()
        .get_mut::<WeaponInventory>(player)
        .unwrap()
        .active_mut()
        .unwrap()
        .ammo = 10;
    app.world_mut()
        .get_mut::<ActionIntent>(player)
        .unwrap()
        .reload_just_pressed = true;
    app.update();
    let before = app
        .world()
        .get::<WeaponInventory>(player)
        .unwrap()
        .reload_timer
        .expect("reload active");

    app.world_mut()
        .get_mut::<ActionIntent>(player)
        .unwrap()
        .swap_slot = Some(0);
    app.update();

    let inv = app.world().get::<WeaponInventory>(player).unwrap();
    let after = inv.reload_timer.expect("reload continues");
    assert!(after < before);
    assert!(messages::<ReloadCancelledMessage>(&app).is_empty());
}

#[test]
fn sprinting_does_not_interrupt_the_reload_timer() {
    let mut app = test_app();
    app.update();
    let player = player_entity(&mut app);

    app.world_mut()
        .get_mut::<WeaponInventory>(player)
        .unwrap()
        .active_mut()
        .unwrap()
        .ammo = 10;
    app.world_mut()
        .get_mut::<ActionIntent>(player)
        .unwrap()
        .reload_just_pressed = true;
    app.update();
    let before = app
        .world()
        .get::<WeaponInventory>(player)
        .unwrap()
        .reload_timer
        .expect("reload active");

    {
        let mut intent = app.world_mut().get_mut::<ActionIntent>(player).unwrap();
        intent.sprint = true;
        intent.move_dir = Vec2::new(1.0, 0.0);
    }
    app.world_mut()
        .get_mut::<Stamina>(player)
        .unwrap()
        .is_sprinting = true;
    step_sim(&mut app, 0.2);

    let inv = app.world().get::<WeaponInventory>(player).unwrap();
    let after = inv.reload_timer.expect("reload continues");
    assert!(after < before);
    assert!(messages::<ReloadCancelledMessage>(&app).is_empty());
}

#[test]
fn death_cancels_reload_and_emits_died_reason_once() {
    let mut app = test_app();
    app.update();
    let player = player_entity(&mut app);

    app.world_mut()
        .get_mut::<WeaponInventory>(player)
        .unwrap()
        .active_mut()
        .unwrap()
        .ammo = 10;
    app.world_mut()
        .get_mut::<ActionIntent>(player)
        .unwrap()
        .reload_just_pressed = true;
    app.update();
    assert!(
        app.world()
            .get::<WeaponInventory>(player)
            .unwrap()
            .reload_timer
            .is_some()
    );

    app.world_mut()
        .write_message(PlayerDiedMessage { survivor: player });
    app.update();

    assert!(
        app.world()
            .get::<WeaponInventory>(player)
            .unwrap()
            .reload_timer
            .is_none()
    );
    let cancelled = messages::<ReloadCancelledMessage>(&app);
    assert_eq!(
        cancelled
            .iter()
            .filter(|msg| msg.reason == ReloadCancelReason::Died)
            .count(),
        1
    );
}

#[test]
fn reload_lifecycle_emits_started_and_finished() {
    let mut app = test_app();
    app.update();
    let player = player_entity(&mut app);

    app.world_mut()
        .get_mut::<WeaponInventory>(player)
        .unwrap()
        .active_mut()
        .unwrap()
        .ammo = 0;
    app.world_mut()
        .get_mut::<ActionIntent>(player)
        .unwrap()
        .fire_held = true;
    app.update();

    let started = messages::<ReloadStartedMessage>(&app);
    assert!(
        started
            .iter()
            .any(|msg| msg.entity == player && msg.duration > 0.0)
    );

    app.world_mut()
        .get_mut::<ActionIntent>(player)
        .unwrap()
        .fire_held = false;

    let reload_secs = app.world().resource::<WeaponRoster>().weapons[0].reload_secs;
    let frames = (reload_secs / (1.0 / 60.0)).ceil() as usize + 5;
    let mut finished_count = 0;
    let mut was_reloading = true;
    for _ in 0..frames {
        app.update();
        let reloading = app
            .world()
            .get::<WeaponInventory>(player)
            .unwrap()
            .reload_timer
            .is_some();
        if was_reloading && !reloading {
            finished_count += messages::<ReloadFinishedMessage>(&app)
                .iter()
                .filter(|msg| msg.entity == player)
                .count();
            was_reloading = false;
        }
    }

    assert_eq!(finished_count, 1);
}

