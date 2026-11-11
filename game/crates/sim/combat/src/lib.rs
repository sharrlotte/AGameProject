use bevy::prelude::*;
use bevy_rapier3d::prelude::PhysicsSet;
use data::{AppRonConfigExt, GameState, SimPostPhysicsSet, SimUpdateSet};

pub mod config;
pub mod projectile;
pub mod reload;
pub mod stance;
pub mod weapon;

pub use config::{WeaponSpecData, WeaponsConfig};
pub use projectile::{Projectile, handle_projectile_collisions, tick_projectiles};
pub use stance::{CombatStance, update_combat_stance};
pub use weapon::{WeaponInventory, WeaponSlotState};

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.register_ron_config::<WeaponsConfig>()
            .add_systems(
                Update,
                (
                    weapon::attach_weapon_inventory,
                    weapon::sync_weapons_hot_reload,
                    weapon::handle_weapon_swap,
                    reload::tick_and_start_reload,
                    weapon::handle_weapon_fire,
                    stance::update_combat_stance,
                    projectile::tick_projectiles,
                )
                    .chain()
                    .in_set(SimUpdateSet)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                PostUpdate,
                (
                    projectile::handle_projectile_collisions
                        .after(PhysicsSet::Writeback)
                        .before(TransformSystems::Propagate),
                    reload::cancel_reload_on_death.after(world::player::apply_player_damage),
                )
                    .in_set(SimPostPhysicsSet)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[cfg(test)]
mod tests;
