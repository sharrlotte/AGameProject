use bevy::prelude::*;

use crate::content::team::Team;

#[derive(Component)]
pub struct Death;

#[derive(Message, Debug, Clone, Copy)]
pub struct ShipDestroyed {
    pub ship: Entity,
    pub team: Team,
}
