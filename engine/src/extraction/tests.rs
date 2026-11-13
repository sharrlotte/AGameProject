use std::time::Duration;

use bevy::prelude::*;
use crate::common::UpgradeKind;
use crate::config::{ConfigHotReloadedMessage, ExtractionConfig};
use crate::message::messages::{
    ExtractionCompletedMessage, PurchaseUpgradeMessage, UpgradesChangedMessage,
};
use crate::message::resources::{RunStats, ScrapBank, UpgradeLevels, UpgradeModifiers};
use crate::world::Survivor;
use crate::extraction::upgrade::{UpgradesConfig, handle_upgrade_purchases};
use crate::extraction::zone::{ExtractionZone, update_extraction_zone};

#[test]
fn extraction_hold_pauses_outside_zone_and_completes_at_duration() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<ExtractionConfig>()
        .init_resource::<RunStats>()
        .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            Duration::from_secs_f32(0.25),
        ))
        .add_message::<ExtractionCompletedMessage>()
        .add_systems(Update, update_extraction_zone);

    let survivor = app
        .world_mut()
        .spawn((Survivor, Transform::from_xyz(0.0, 0.9, -19.5)))
        .id();
    app.world_mut().spawn((
        ExtractionZone {
            radius: 3.2,
            hold_duration_secs: 4.0,
            progress_secs: 0.0,
            active: false,
            completed: false,
        },
        Transform::from_xyz(0.0, 0.0, -19.5),
    ));

    app.update();
    for _ in 0..8 {
        app.update();
    }
    assert!((app.world().resource::<RunStats>().extraction_progress_secs - 2.0).abs() < 0.01);

    app.world_mut()
        .get_mut::<Transform>(survivor)
        .unwrap()
        .translation = Vec3::new(0.0, 0.9, 0.0);
    for _ in 0..4 {
        app.update();
    }
    assert!((app.world().resource::<RunStats>().extraction_progress_secs - 2.0).abs() < 0.01);

    app.world_mut()
        .get_mut::<Transform>(survivor)
        .unwrap()
        .translation = Vec3::new(0.0, 0.9, -19.5);
    for _ in 0..8 {
        app.update();
    }
    assert!((app.world().resource::<RunStats>().extraction_progress_secs - 4.0).abs() < 0.01);

    let msgs = app
        .world()
        .resource::<Messages<ExtractionCompletedMessage>>();
    let mut cursor = msgs.get_cursor();
    let completed: Vec<_> = cursor.read(msgs).copied().collect();
    assert_eq!(completed.len(), 1);
    assert_eq!(completed[0].survivor, survivor);
}

#[test]
fn upgrade_purchase_deducts_scrap_and_updates_modifiers() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<UpgradesConfig>()
        .insert_resource(ScrapBank { banked: 25 })
        .init_resource::<UpgradeLevels>()
        .init_resource::<UpgradeModifiers>()
        .add_message::<PurchaseUpgradeMessage>()
        .add_message::<UpgradesChangedMessage>()
        .add_message::<ConfigHotReloadedMessage>()
        .add_systems(Update, handle_upgrade_purchases);

    app.world_mut().write_message(PurchaseUpgradeMessage {
        kind: UpgradeKind::WeaponDamage,
    });
    app.update();

    assert_eq!(app.world().resource::<ScrapBank>().banked, 15);
    assert_eq!(app.world().resource::<UpgradeLevels>().weapon_damage, 1);
    assert!((app.world().resource::<UpgradeModifiers>().damage_multiplier - 1.15).abs() < 0.001);
}

