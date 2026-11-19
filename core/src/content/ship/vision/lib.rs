use bevy::prelude::*;

use crate::content::ship::ai::TacticalBoard;
use crate::content::team::{DEFAULT_TEAM_ID, Team};

pub fn player_detects(board: &TacticalBoard, entity: Entity) -> bool {
    board
        .teams
        .get(&DEFAULT_TEAM_ID)
        .is_some_and(|plan| plan.intel.contacts.contains_key(&entity))
}

pub fn visible_to_player(board: &TacticalBoard, team: &Team, entity: Entity) -> bool {
    team.is_player() || player_detects(board, entity)
}
