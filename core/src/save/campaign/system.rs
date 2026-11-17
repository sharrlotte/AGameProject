use bevy::prelude::*;

use super::component::CampaignSave;
use super::lib::load_campaign;

pub fn init_campaign_save(mut campaign: ResMut<CampaignSave>) {
    *campaign = load_campaign();
}
