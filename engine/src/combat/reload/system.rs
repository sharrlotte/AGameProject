use bevy::prelude::*;
use crate::message::messages::{
    PlayerDiedMessage, ReloadCancelReason, ReloadCancelledMessage, ReloadFinishedMessage,
    ReloadStartedMessage,
};
use crate::message::resources::UpgradeModifiers;
use crate::world::{ActionIntent, Survivor};

use crate::combat::stance::state::CombatStance;
use crate::combat::weapon::state::WeaponInventory;

/// Begins a reload on `inventory` and emits [`ReloadStartedMessage`].
pub fn start_reload(
    inventory: &mut WeaponInventory,
    base_reload_secs: f32,
    reload_speed: f32,
    entity: Entity,
    started_writer: &mut MessageWriter<ReloadStartedMessage>,
) {
    let duration = base_reload_secs / reload_speed;
    inventory.reload_total = duration;
    inventory.reload_timer = Some(duration);
    started_writer.write(ReloadStartedMessage { entity, duration });
}

/// Advances active reload timers and handles manual reload triggers.
pub fn tick_and_start_reload(
    time: Res<Time>,
    upgrades: Res<UpgradeModifiers>,
    mut started_writer: MessageWriter<ReloadStartedMessage>,
    mut finished_writer: MessageWriter<ReloadFinishedMessage>,
    mut shooters: Query<(Entity, &mut ActionIntent, &mut WeaponInventory), With<Survivor>>,
) {
    let dt = time.delta_secs();
    let reload_speed = upgrades.reload_speed_multiplier.max(0.1);

    for (entity, mut intent, mut inventory) in &mut shooters {
        if let Some(remaining) = inventory.reload_timer.as_mut() {
            *remaining -= dt;
            if *remaining <= 0.0 {
                if let Some(active) = inventory.active_mut() {
                    active.ammo = active.spec.mag_size;
                }
                inventory.reload_timer = None;
                inventory.reload_total = 0.0;
                finished_writer.write(ReloadFinishedMessage { entity });
            }
        }

        let want_reload = intent.reload_just_pressed;
        intent.reload_just_pressed = false;
        if want_reload
            && inventory.reload_timer.is_none()
            && let Some(active) = inventory.active()
            && active.ammo < active.spec.mag_size
        {
            let base_reload_secs = active.spec.reload_secs;
            start_reload(
                &mut inventory,
                base_reload_secs,
                reload_speed,
                entity,
                &mut started_writer,
            );
        }
    }
}

/// Cancels an in-flight reload when the survivor dies and drops the stance.
pub fn cancel_reload_on_death(
    mut died_reader: MessageReader<PlayerDiedMessage>,
    mut cancelled_writer: MessageWriter<ReloadCancelledMessage>,
    mut inventories: Query<&mut WeaponInventory>,
    mut stances: Query<&mut CombatStance>,
) {
    for msg in died_reader.read() {
        if let Ok(mut inventory) = inventories.get_mut(msg.survivor)
            && inventory.reload_timer.is_some()
        {
            inventory.reload_timer = None;
            inventory.reload_total = 0.0;
            cancelled_writer.write(ReloadCancelledMessage {
                entity: msg.survivor,
                reason: ReloadCancelReason::Died,
            });
        }
        if let Ok(mut stance) = stances.get_mut(msg.survivor) {
            stance.aiming = false;
            stance.hold_timer = 0.0;
        }
    }
}

