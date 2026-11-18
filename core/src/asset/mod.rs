#[cfg(feature = "dev")]
pub mod dev {
    mod component;
    mod lib;
    mod system;

    use bevy::prelude::*;
    use system::validate_assets;

    pub struct AssetCheckPlugin;

    impl Plugin for AssetCheckPlugin {
        fn build(&self, app: &mut App) {
            app.add_systems(Startup, validate_assets);
        }
    }
}
