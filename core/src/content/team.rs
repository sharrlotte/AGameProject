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
// bits 16–31 reserved for block hitbox per-team

pub const DEFAULT_TEAM_ID: u8 = 0;

#[derive(Component, Debug, Clone, Copy, Reflect, Serialize, Deserialize)]
pub struct Team {
    pub id: u8,
    color: Color,
}

impl Default for Team {
    fn default() -> Self {
        Self::new(DEFAULT_TEAM_ID)
    }
}

impl PartialEq for Team {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Team {
    pub fn new(id: u8) -> Self {
        assert!(id < 16, "Team id must be < 16");
        Self {
            id,
            color: COLORS[id as usize],
        }
    }

    pub fn is_player(&self) -> bool {
        self.id == DEFAULT_TEAM_ID
    }

    pub fn color(&self, multiplier: f32) -> Color {
        if multiplier == 1.0 {
            return self.color;
        }

        let mut rgba = self.color.to_srgba();
        rgba.red *= multiplier;
        rgba.green *= multiplier;
        rgba.blue *= multiplier;

        return Color::srgba(rgba.red, rgba.green, rgba.blue, rgba.alpha);
    }

    pub fn ship_group(&self) -> Group {
        GROUP_SHIP
    }

    pub fn ship_filter(&self) -> Group {
        GROUP_SHIP | GROUP_RADAR | GROUP_DEBRIS
    }

    pub fn block_hitbox_group(&self) -> Group {
        Group::from_bits_truncate(1 << (16 + self.id))
    }

    pub fn block_hitbox_filter(&self) -> Group {
        GROUP_BULLET
    }

    pub fn bullet_group(&self) -> Group {
        GROUP_BULLET
    }

    pub fn bullet_filter(&self) -> Group {
        self.enemy_block_hitbox_filter()
    }

    pub fn radar_group(&self) -> Group {
        GROUP_RADAR
    }

    pub fn radar_filter(&self) -> Group {
        GROUP_SHIP
    }

    fn enemy_block_hitbox_filter(&self) -> Group {
        let all_blocks = 0xFFFF_0000;
        let my_block = 1 << (16 + self.id);
        Group::from_bits_truncate(all_blocks & !my_block)
    }
}
