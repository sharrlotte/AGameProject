use bevy::prelude::*;

use crate::content::team::Team;

pub fn is_selectable(team: &Team, visibility: &Visibility) -> bool {
    team.is_player() && *visibility != Visibility::Hidden
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hidden_enemy_is_not_selectable() {
        assert!(!is_selectable(&Team::new(1), &Visibility::Hidden));
        assert!(!is_selectable(&Team::new(1), &Visibility::Visible));
    }

    #[test]
    fn visible_player_ship_is_selectable() {
        assert!(is_selectable(&Team::new(0), &Visibility::Visible));
        assert!(!is_selectable(&Team::new(0), &Visibility::Hidden));
    }
}
