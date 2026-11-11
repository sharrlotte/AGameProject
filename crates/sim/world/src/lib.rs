pub mod arena;
pub mod config;
pub mod intent;
pub mod player;
#[cfg(feature = "dev")]
pub mod testanim;

pub use arena::{ArenaGround, ArenaObstacle, ArenaPlugin};
pub use config::{ArenaConfig, ObstacleDef, PlayerConfig};
pub use intent::ActionIntent;
pub use player::{
    AimDirection, Health, LocalPlayer, MovementStats, PlayerPlugin, Stamina, Survivor,
};
#[cfg(feature = "dev")]
pub use testanim::AnimTestPlugin;
