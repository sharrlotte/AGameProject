pub mod state;
pub mod system;

pub use state::{Enemy, FacingDirection};
pub use system::{enemy_chase_and_attack, freeze_enemies_on_game_over, sync_enemies_hot_reload};
