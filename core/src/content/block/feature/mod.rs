pub mod ammo;
pub mod component;
pub mod power;
pub mod shield;
pub mod sprite;
pub mod thruster;
pub mod turret;

pub use component::*;

use bevy::prelude::*;

use ammo::AmmoPlugin;
use power::PowerPlugin;
use shield::ShieldPlugin;
use thruster::ThrusterPlugin;
use turret::TurretPlugin;

pub struct ComponentPlugin;

impl Plugin for ComponentPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ThrusterPlugin,
            TurretPlugin,
            PowerPlugin,
            AmmoPlugin,
            ShieldPlugin,
        ));
    }
}
