use bevy::prelude::*;

use crate::content::block::radar::system::radar_scan;

pub struct RadarPlugin;

impl Plugin for RadarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, radar_scan);
    }
}
