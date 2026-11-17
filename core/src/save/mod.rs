pub mod battle;
pub mod campaign;
pub mod design;
pub mod document;

use bevy::prelude::*;

use battle::BattleSavePlugin;
use campaign::CampaignSavePlugin;
use design::DesignSavePlugin;

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((DesignSavePlugin, CampaignSavePlugin, BattleSavePlugin));
    }
}
