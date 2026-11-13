use bevy::prelude::*;
use crate::world::{ActionIntent, Health, Survivor};

use crate::config::WeaponsConfig;
use crate::combat::weapon::state::WeaponInventory;

use super::state::CombatStance;

/// Derives the combat stance from aim, fire, and reload state.
///
/// The hold timer keeps the arms raised between shots and after the reload ends,
/// so the upper-body layer never flickers. Dead players are never in stance.
pub fn update_combat_stance(
    time: Res<Time>,
    cfg: Res<WeaponsConfig>,
    mut stances: Query<
        (&ActionIntent, &WeaponInventory, &Health, &mut CombatStance),
        With<Survivor>,
    >,
) {
    let dt = time.delta_secs();
    let hold = cfg.stance_hold_secs.max(0.0);

    for (intent, inventory, health, mut stance) in &mut stances {
        if health.current <= 0.0 {
            stance.aiming = false;
            stance.hold_timer = 0.0;
            continue;
        }

        // Reload owns the arms for its whole duration, independent of movement.
        if intent.aim_held || intent.fire_held || inventory.reload_timer.is_some() {
            stance.hold_timer = hold;
            stance.aiming = true;
        } else if stance.hold_timer > 0.0 {
            stance.hold_timer = (stance.hold_timer - dt).max(0.0);
            stance.aiming = stance.hold_timer > 0.0;
        } else {
            stance.aiming = false;
        }
    }
}

