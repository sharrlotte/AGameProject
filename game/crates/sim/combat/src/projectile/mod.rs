pub mod state;
pub mod system;

pub use state::Projectile;
pub use system::{handle_projectile_collisions, spawn_projectile, tick_projectiles};
