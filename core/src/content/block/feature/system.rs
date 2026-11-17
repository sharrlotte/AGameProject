use crate::content::block::feature::{
    ammo::AmmoPlugin, power::PowerPlugin, thruster::ThrusterPlugin, turret::TurretPlugin,
};
use bevy::prelude::*;

pub struct ComponentPlugin;

impl Plugin for ComponentPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ThrusterPlugin, TurretPlugin, PowerPlugin, AmmoPlugin));
    }
}
