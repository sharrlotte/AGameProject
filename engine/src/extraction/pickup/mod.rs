pub mod state;
pub mod system;

pub use state::ScrapPickup;
pub use system::{collect_scrap_pickups, spawn_scrap_on_enemy_killed};

