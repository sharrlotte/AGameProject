pub mod state;
pub mod system;

pub use state::{ACID_MAX_FLIGHT_SECS, ACID_PROJECTILE_RADIUS, AcidProjectile, Ranged};
pub use system::{
    enemy_ranged, fall_acid_projectiles, launch_velocity, resolve_acid_projectiles,
};

#[cfg(test)]
mod tests;
