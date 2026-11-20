use std::fmt;

use bevy::prelude::*;
use bevy_rapier2d::prelude::Group;
use serde::{Deserialize, Serialize};

const COLORS: [Color; 16] = [
    Color::srgb(0.0, 1.0, 0.0),
    Color::srgb(1.0, 0.0, 0.0),
    Color::srgb(0.0, 0.0, 1.0),
    Color::srgb(1.0, 1.0, 0.0),
    Color::srgb(1.0, 0.5, 0.0),
    Color::srgb(0.5, 0.0, 0.5),
    Color::srgb(0.0, 1.0, 1.0),
    Color::srgb(1.0, 0.0, 1.0),
    Color::srgb(0.75, 1.0, 0.0),
    Color::srgb(1.0, 0.75, 0.8),
    Color::srgb(0.0, 0.5, 0.5),
    Color::srgb(0.6, 0.4, 0.2),
    Color::srgb(0.5, 0.5, 0.5),
    Color::srgb(1.0, 1.0, 1.0),
    Color::srgb(0.0, 0.0, 0.0),
    Color::srgb(0.75, 0.75, 0.75),
];

pub const GROUP_SHIP: Group = Group::GROUP_1;
pub const GROUP_BULLET: Group = Group::GROUP_2;
pub const GROUP_RADAR: Group = Group::GROUP_3;
pub const GROUP_DEBRIS: Group = Group::GROUP_4;

pub const DEFAULT_TEAM_ID: u8 = 0;
const MAX_TEAM_ID: u8 = 15;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidTeam(u8);

impl fmt::Display for InvalidTeam {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "team id {} is out of range (0..={MAX_TEAM_ID})", self.0)
    }
}

impl std::error::Error for InvalidTeam {}

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(try_from = "u8", into = "u8")]
pub struct Team(u8);

impl Default for Team {
    fn default() -> Self {
        Self(DEFAULT_TEAM_ID)
    }
}

impl TryFrom<u8> for Team {
    type Error = InvalidTeam;

    fn try_from(id: u8) -> Result<Self, Self::Error> {
        if id <= MAX_TEAM_ID {
            Ok(Self(id))
        } else {
            Err(InvalidTeam(id))
        }
    }
}

impl From<Team> for u8 {
    fn from(team: Team) -> Self {
        team.0
    }
}

impl Team {
    pub fn new(id: u8) -> Self {
        Self::try_from(id).unwrap_or_else(|error| panic!("{error}"))
    }

    pub fn id(self) -> u8 {
        self.0
    }

    pub fn is_player(self) -> bool {
        self.0 == DEFAULT_TEAM_ID
    }

    pub fn color(self, multiplier: f32) -> Color {
        let base = COLORS[self.0 as usize];
        if multiplier == 1.0 {
            return base;
        }
        let mut rgba = base.to_srgba();
        rgba.red *= multiplier;
        rgba.green *= multiplier;
        rgba.blue *= multiplier;
        Color::srgba(rgba.red, rgba.green, rgba.blue, rgba.alpha)
    }

    pub fn ship_group(self) -> Group {
        GROUP_SHIP
    }

    pub fn ship_filter(self) -> Group {
        GROUP_SHIP | GROUP_RADAR | GROUP_DEBRIS
    }

    pub fn block_hitbox_group(self) -> Group {
        Group::from_bits_truncate(1_u32 << (16 + self.0))
    }

    pub fn block_hitbox_filter(self) -> Group {
        GROUP_BULLET
    }

    pub fn bullet_group(self) -> Group {
        GROUP_BULLET
    }

    pub fn bullet_filter(self) -> Group {
        self.enemy_block_hitbox_filter()
    }

    pub fn radar_group(self) -> Group {
        GROUP_RADAR
    }

    pub fn radar_filter(self) -> Group {
        GROUP_SHIP
    }

    fn enemy_block_hitbox_filter(self) -> Group {
        let all_blocks = 0xFFFF_0000;
        let my_block = 1_u32 << (16 + self.0);
        Group::from_bits_truncate(all_blocks & !my_block)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_team_id_is_rejected() {
        assert!(Team::try_from(16).is_err());
        assert_eq!(Team::try_from(15).map(Team::id), Ok(15));
    }

    #[test]
    fn serde_rejects_out_of_range_team() {
        assert!(ron::from_str::<Team>("16").is_err());
        assert_eq!(ron::from_str::<Team>("1").map(Team::id), Ok(1));
    }
}
