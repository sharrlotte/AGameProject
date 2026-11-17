use bevy::prelude::*;

use crate::content::team::Team;

#[derive(Message, Debug, Clone, Copy)]
pub struct ShipDestroyed {
    pub ship: Entity,
    pub team: Team,
}
