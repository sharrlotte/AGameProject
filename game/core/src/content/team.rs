use bevy::prelude::*;
use bevy_rapier2d::prelude::Group;
use serde::{Deserialize, Serialize};

const COLORS: [Color; 16] = [
    Color::srgb(0.0, 1.0, 0.0),    // green
    Color::srgb(1.0, 0.0, 0.0),    // red
    Color::srgb(0.0, 0.0, 1.0),    // blue
    Color::srgb(1.0, 1.0, 0.0),    // yellow
    Color::srgb(1.0, 0.5, 0.0),    // orange
    Color::srgb(0.5, 0.0, 0.5),    // purple
    Color::srgb(0.0, 1.0, 1.0),    // cyan
    Color::srgb(1.0, 0.0, 1.0),    // magenta
    Color::srgb(0.75, 1.0, 0.0),   // lime
    Color::srgb(1.0, 0.75, 0.8),   // pink
    Color::srgb(0.0, 0.5, 0.5),    // teal
    Color::srgb(0.6, 0.4, 0.2),    // brown
    Color::srgb(0.5, 0.5, 0.5),    // gray
    Color::srgb(1.0, 1.0, 1.0),    // white
    Color::srgb(0.0, 0.0, 0.0),    // black
    Color::srgb(0.75, 0.75, 0.75), // silver
];

#[derive(Component, Debug, Clone, Copy, Reflect, Serialize, Deserialize)]
pub struct Team {
    pub id: u8,
    color: Color,
}

impl PartialEq for Team {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Default for Team {
    fn default() -> Self {
        Self {
            id: 0,
            color: COLORS[0],
        }
    }
}
impl Team {
    pub fn is_player(&self) -> bool {
        self.id == 0
    }
}
// Collision Groups
// Group 1: Hulls (Physical) - All ships collide with each other.
// Group 16-31: Team Hitboxes (Sensors) - For detecting damage.

pub const GROUP_HULL: Group = Group::GROUP_1;
pub const GROUP_BULLET: Group = Group::GROUP_2;
pub const GROUP_RADAR: Group = Group::GROUP_3;

impl Team {
    pub fn new(id: u8) -> Self {
        // Limit to 16 teams (0-15) because we use bits 16-31 for hitboxes.
        assert!(id < 16, "Team ID must be less than 16");
        Self {
            id,
            color: COLORS[id as usize],
        }
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

    /// Group for Ship Hulls (Physical)
    pub fn get_hull_group(&self) -> Group {
        GROUP_HULL
    }

    /// Filter for Ship Hulls (Interact with other Hulls)
    pub fn get_hull_filter(&self) -> Group {
        GROUP_HULL
    }

    /// Group for Blocks (Hurtboxes)
    pub fn get_block_group(&self) -> Group {
        // Shift 16 bits + id
        Group::from_bits_truncate(1 << (16 + self.id))
    }

    /// Filter for Blocks (Hit by Bullets and Radar)
    pub fn get_block_filter(&self) -> Group {
        GROUP_BULLET | GROUP_RADAR
    }

    /// Group for Bullets
    pub fn get_bullet_group(&self) -> Group {
        GROUP_BULLET
    }

    /// Filter for Bullets (Hit Enemy Hurtboxes)
    pub fn get_bullet_filter(&self) -> Group {
        self.get_enemy_hitbox_filter()
    }

    /// Group for Radar
    pub fn get_radar_group(&self) -> Group {
        GROUP_RADAR
    }

    /// Filter for Radar (Detect Enemy Hurtboxes)
    pub fn get_radar_filter(&self) -> Group {
        self.get_enemy_hitbox_filter()
    }

    /// Helper to get all enemy hitboxes
    pub fn get_enemy_hitbox_filter(&self) -> Group {
        // Hit all hitboxes EXCEPT our own team's hitbox.
        let all_hitboxes = 0xFFFF0000; // Bits 16-31
        let my_hitbox = 1 << (16 + self.id);
        Group::from_bits_truncate(all_hitboxes & !my_hitbox)
    }
}
