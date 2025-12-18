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

// Collision Groups
// Group 1: Hulls (Physical) - All ships collide with each other.
// Group 16-31: Team Hitboxes (Sensors) - For detecting damage.

pub const GROUP_HULL: Group = Group::GROUP_1;

impl Team {
    pub fn new(id: u8) -> Self {
        // Limit to 16 teams (0-15) because we use bits 16-31 for hitboxes.
        assert!(id < 16, "Team ID must be less than 16");
        Self { id }
    }

    pub fn get_hull_group(&self) -> Group {
        GROUP_HULL
    }

    pub fn get_hull_filter(&self) -> Group {
        // Hulls collide with all other Hulls
        GROUP_HULL
    }

    pub fn get_hitbox_group(&self) -> Group {
        // Shift 16 bits + id
        Group::from_bits_truncate(1 << (16 + self.id))
    }

    pub fn get_bullet_filter(&self) -> Group {
        // Hit all hitboxes EXCEPT our own team's hitbox.
        // Also can add WALLS or other things here.
        let all_hitboxes = 0xFFFF0000; // Bits 16-31
        let my_hitbox = 1 << (16 + self.id);
        Group::from_bits_truncate(all_hitboxes & !my_hitbox)
    }

    pub fn get_enemy_hitbox_filter(&self) -> Group {
        // Hit all hitboxes EXCEPT our own team's hitbox.
        let all_hitboxes = 0xFFFF0000; // Bits 16-31
        let my_hitbox = 1 << (16 + self.id);
        Group::from_bits_truncate(all_hitboxes & !my_hitbox)
    }
}
