pub mod battle;
pub mod campaign;
pub mod design;
pub mod document;
pub mod fleet;

use bevy::prelude::*;

use battle::BattleSavePlugin;
use campaign::CampaignSavePlugin;
use design::DesignSavePlugin;
use fleet::FleetSavePlugin;

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            DesignSavePlugin,
            CampaignSavePlugin,
            BattleSavePlugin,
            FleetSavePlugin,
        ));
    }
}
