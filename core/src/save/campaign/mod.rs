pub mod component;
pub mod lib;
pub mod system;

pub use component::*;
pub use lib::*;
pub use system::*;

use bevy::prelude::*;

pub struct CampaignSavePlugin;

impl Plugin for CampaignSavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CampaignSave>()
            .add_systems(PreStartup, init_campaign_save);
    }
}
