#[cfg(feature = "dev")]
use bevy::prelude::*;

use crate::content::team::Team;

#[cfg(feature = "dev")]
use super::InspectedShip;

#[cfg(feature = "dev")]
pub fn is_inspected(inspected: &InspectedShip, entity: Entity) -> bool {
    inspected.0 == Some(entity)
}

pub fn can_inspect_team(team: &Team) -> bool {
    #[cfg(feature = "dev")]
    {
        let _ = team;
        true
    }
    #[cfg(not(feature = "dev"))]
    {
        team.is_player()
    }
}
