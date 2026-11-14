pub mod state;
pub mod system;

pub use state::{HitLog, Projectile};
pub use system::{handle_projectile_collisions, spawn_projectile, tick_projectiles};

#[cfg(test)]
mod tests;

