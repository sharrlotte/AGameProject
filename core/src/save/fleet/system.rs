use bevy::prelude::*;

use crate::resource::DeployPayload;

use super::lib::write_fleet;

pub fn save_fleet_on_change(deploy: Res<DeployPayload>) {
    if !deploy.is_changed() {
        return;
    }

    if let Err(error) = write_fleet(&deploy.fleet) {
        warn!("Failed to save deployed fleet: {error}");
    }
}
