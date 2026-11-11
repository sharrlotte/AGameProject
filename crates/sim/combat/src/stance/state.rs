use bevy::prelude::*;

/// Simulation-owned combat stance consumed by animation and future gameplay modifiers.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct CombatStance {
    pub aiming: bool,
    /// Remaining seconds the stance stays raised after the last triggering action ends.
    pub hold_timer: f32,
}
