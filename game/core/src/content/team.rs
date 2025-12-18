use bevy::prelude::*;
use bevy_rapier2d::prelude::Group;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub struct Team {
    pub id: u8,
}

impl Default for Team {
    fn default() -> Self {
        Self { id: 0 }
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
        Self { id }
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
