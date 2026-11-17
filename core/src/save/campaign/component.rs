use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::save::document::DocumentError;

use super::lib::save_campaign;

pub const CAMPAIGN_SCHEMA_VERSION: u32 = 1;

#[derive(Resource, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CampaignSave {
    pub version: u32,
    pub current_mission: String,
    pub completed_missions: Vec<String>,
}

impl Default for CampaignSave {
    fn default() -> Self {
        Self {
            version: CAMPAIGN_SCHEMA_VERSION,
            current_mission: "quick-battle".to_string(),
            completed_missions: Vec::new(),
        }
    }
}

impl CampaignSave {
    pub fn record_victory(&mut self, mission: &str) -> Result<(), DocumentError> {
        if !self.completed_missions.iter().any(|m| m == mission) {
            self.completed_missions.push(mission.to_string());
        }
        save_campaign(self)
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

crate::ron_document!(CampaignSave, CAMPAIGN_SCHEMA_VERSION, "campaign save");
