use serde::{Deserialize, Serialize};

use crate::content::block::BlockRegistry;
use super::block::BlockSnapshot;
use super::ship::{TransformSave, VelocitySave};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DebrisSnapshot {
    pub team: u8,
    pub transform: TransformSave,
    pub velocity: VelocitySave,
    pub remaining_secs: f32,
    pub blocks: Vec<BlockSnapshot>,
}

impl DebrisSnapshot {
    pub fn validate(&self, registry: &BlockRegistry) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if self.team >= 16 {
            errors.push(format!("debris has invalid team {}", self.team));
        }
        if !self.remaining_secs.is_finite() || self.remaining_secs < 0.0 {
            errors.push(format!("debris has invalid lifetime {}", self.remaining_secs));
        }
        errors.extend(
            BlockSnapshot::validate_blocks(&self.blocks, registry, false)
                .err()
                .unwrap_or_default(),
        );
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
