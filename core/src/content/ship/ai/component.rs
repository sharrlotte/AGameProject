use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub enum Doctrine {
    #[default]
    Auto,
    Brawler,
    Skirmisher,
    Sniper,
    Support,
    Escort,
}

impl Doctrine {
    pub const ALL: [Self; 6] = [
        Self::Auto,
        Self::Brawler,
        Self::Skirmisher,
        Self::Sniper,
        Self::Support,
        Self::Escort,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::Brawler => "Brawler",
            Self::Skirmisher => "Skirmisher",
            Self::Sniper => "Sniper",
            Self::Support => "Support",
            Self::Escort => "Escort",
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Skill(u8);

impl Default for Skill {
    fn default() -> Self {
        Self(5)
    }
}

impl Skill {
    pub fn new(level: u8) -> Self {
        Self(level.clamp(1, 10))
    }

    pub fn level(self) -> u8 {
        self.0.clamp(1, 10)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Reflect)]
pub enum Role {
    #[default]
    Brawler,
    Skirmisher,
    Sniper,
    Support,
    Escort,
}

impl Role {
    pub fn label(self) -> &'static str {
        match self {
            Self::Brawler => "Brawler",
            Self::Skirmisher => "Skirmisher",
            Self::Sniper => "Sniper",
            Self::Support => "Support",
            Self::Escort => "Escort",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub struct FixedMount {
    pub mount_angle: f32,
    pub range: f32,
    pub dps: f32,
    pub aim_tolerance: f32,
    pub projectile_speed: f32,
}

#[derive(Component, Debug, Clone, Default, Reflect)]
#[reflect(Component)]
pub struct Capability {
    pub role: Role,
    pub max_range: f32,
    pub optimal_range: f32,
    pub max_radar_range: f32,
    pub offensive_dps: f32,
    pub support_weight: f32,
    pub fixed_mount: Option<FixedMount>,
    pub fixed_dps_ratio: f32,
    pub has_rotating_turrets: bool,
    pub has_laser_beam: bool,
    pub has_pulling_beam: bool,
    pub has_point_defense: bool,
    pub uses_energy_weapons: bool,
    pub uses_ammo_weapons: bool,
    pub booster_count: usize,
    pub booster_radius: f32,
    pub booster_rate: f32,
    pub thrust_to_mass: f32,
    pub can_move: bool,
    pub quadrant_hp: [f32; 4],
    pub drain_rate: f32,
}

#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct Readiness {
    pub hp_ratio: f32,
    pub energy_ratio: f32,
    pub ammo_ratio: f32,
    pub disarmed: bool,
    pub firing_drain_beam: bool,
    pub power_supply: f32,
    pub battery_ratio: f32,
}

impl Default for Readiness {
    fn default() -> Self {
        Self {
            hp_ratio: 1.0,
            energy_ratio: 1.0,
            ammo_ratio: 1.0,
            disarmed: false,
            firing_drain_beam: false,
            power_supply: 1.0,
            battery_ratio: 1.0,
        }
    }
}

#[derive(Component, Debug, Clone, Default)]
pub struct Sensor {
    pub local_contacts: Vec<Entity>,
}

#[derive(Debug, Clone, Copy)]
pub struct SharedContact {
    pub entity: Entity,
    pub position: Vec2,
    pub velocity: Vec2,
    pub estimated_threat: f32,
    pub has_fixed_gun: bool,
    pub heading: Vec2,
}

#[derive(Debug, Clone, Copy)]
pub struct LastSeen {
    pub position: Vec2,
    pub velocity: Vec2,
    pub ttl: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct TeamAlert {
    pub position: Vec2,
    pub ttl: f32,
}

#[derive(Debug, Clone, Default)]
pub struct TeamIntel {
    pub contacts: HashMap<Entity, SharedContact>,
    pub last_known: HashMap<Entity, LastSeen>,
    pub alerts: Vec<TeamAlert>,
}

#[derive(Debug, Clone, Default)]
pub struct TeamPlan {
    pub intel: TeamIntel,
    pub focus_target: Option<Entity>,
    pub vip_ally: Option<Entity>,
    pub rally_point: Vec2,
    pub ally_count: usize,
}

#[derive(Resource, Default, Debug)]
pub struct TacticalBoard {
    pub teams: HashMap<u8, TeamPlan>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Serialize, Deserialize)]
pub enum Posture {
    SpinalAim,
    Orbit,
    Kite,
    Brawl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Serialize, Deserialize)]
pub enum SupportKind {
    Overdrive,
    Screen,
}

#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub enum Tactic {
    Engage {
        target: Entity,
        range: f32,
        posture: Posture,
    },
    Recharge {
        threat: Entity,
        safe_range: f32,
    },
    Retreat {
        threat: Entity,
    },
    Support {
        ally: Entity,
        kind: SupportKind,
    },
    Regroup {
        position: Vec2,
    },
    Investigate {
        position: Vec2,
    },
    Search {
        waypoint: Vec2,
    },
}

impl Tactic {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Engage { .. } => "Engage",
            Self::Recharge { .. } => "Recharge",
            Self::Retreat { .. } => "Retreat",
            Self::Support { .. } => "Support",
            Self::Regroup { .. } => "Regroup",
            Self::Investigate { .. } => "Investigate",
            Self::Search { .. } => "Search",
        }
    }

    pub fn same_kind(&self, other: &Self) -> bool {
        matches!(
            (self, other),
            (Self::Engage { .. }, Self::Engage { .. })
                | (Self::Recharge { .. }, Self::Recharge { .. })
                | (Self::Retreat { .. }, Self::Retreat { .. })
                | (Self::Support { .. }, Self::Support { .. })
                | (Self::Regroup { .. }, Self::Regroup { .. })
                | (Self::Investigate { .. }, Self::Investigate { .. })
                | (Self::Search { .. }, Self::Search { .. })
        )
    }

    pub fn target_entity(&self) -> Option<Entity> {
        match *self {
            Self::Engage { target, .. }
            | Self::Recharge { threat: target, .. }
            | Self::Retreat { threat: target }
            | Self::Support { ally: target, .. } => Some(target),
            _ => None,
        }
    }
}

#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct Brain {
    pub current: Tactic,
    pub spawn_position: Vec2,
    pub think_timer: Timer,
    pub strafe_sign: f32,
    pub strafe_timer: f32,
}

impl Brain {
    pub fn new(spawn_position: Vec2, think_rate_hz: f32) -> Self {
        Self {
            current: Tactic::Search {
                waypoint: spawn_position,
            },
            spawn_position,
            think_timer: Timer::from_seconds(
                1.0 / think_rate_hz.max(0.1),
                TimerMode::Repeating,
            ),
            strafe_sign: 1.0,
            strafe_timer: 2.0,
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component)]
pub struct Steering {
    pub desired_heading: Option<f32>,
    pub thrust_world: Vec2,
    pub throttle_cap: f32,
}

impl Default for Steering {
    fn default() -> Self {
        Self {
            desired_heading: None,
            thrust_world: Vec2::ZERO,
            throttle_cap: 1.0,
        }
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Reflect)]
#[reflect(Component)]
pub struct FireOrder {
    pub primary_target: Option<Entity>,
    pub lead_accuracy: f32,
}

impl Default for FireOrder {
    fn default() -> Self {
        Self {
            primary_target: None,
            lead_accuracy: 1.0,
        }
    }
}
