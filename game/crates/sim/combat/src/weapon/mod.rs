pub mod state;
pub mod system;

pub use state::{WeaponInventory, WeaponSlotState};
pub use system::{
    attach_weapon_inventory, handle_weapon_fire, handle_weapon_swap, sync_weapons_hot_reload,
};
