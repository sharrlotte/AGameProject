use serde::Deserialize;

/// Hard upper bound on unique enemies a single projectile can register.
pub const MAX_HITS: usize = 16;

/// Smallest legal enemy piercing cost, used to bound authored weapon piercing power.
pub const MIN_PIERCE_COST: f32 = 0.25;

/// Weapon piercing profile governing multi-body bullet traversal.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct PierceConfig {
    #[serde(default)]
    pub power: f32,
    #[serde(default)]
    pub penalty: f32,
}

impl Default for PierceConfig {
    fn default() -> Self {
        Self {
            power: 0.0,
            penalty: 0.0,
        }
    }
}
