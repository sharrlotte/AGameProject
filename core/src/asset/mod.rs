#[cfg(feature = "dev")]
mod component;
#[cfg(feature = "dev")]
mod lib;
#[cfg(feature = "dev")]
mod system;

use bevy::prelude::*;

#[cfg(feature = "dev")]
use system::validate_assets;

pub struct AssetCheckPlugin;

impl Plugin for AssetCheckPlugin {
    fn build(&self, _app: &mut App) {
        #[cfg(feature = "dev")]
        app.add_systems(Startup, validate_assets);
    }
}
