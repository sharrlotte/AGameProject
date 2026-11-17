use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::constant::AI_THINK_RATE_HZ;

/// Authorable AI profile stored inline on a `ShipDesign` and mirrored on the
/// spawned ship entity. Rules are evaluated in priority order at the shared
/// `AI_THINK_RATE_HZ`.
#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiProfile {
    pub initial: BehaviorSpec,
    #[serde(default)]
    pub rules: Vec<AiRule>,
}

impl Default for AiProfile {
    fn default() -> Self {
        Self {
            initial: BehaviorSpec::Idle,
            rules: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiRule {
    pub priority: i32,
    pub when: AiCondition,
    pub then: BehaviorSpec,
    #[serde(default)]
    pub cooldown_secs: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AiCondition {
    Always,
    HpBelow(f32),
    HpAbove(f32),
    EnemyInRange(f32),
    AllyCountBelow(u32),
    AllyCountAbove(u32),
    TargetLost,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BehaviorSpec {
    Idle,
    Wander { radius: f32 },
    Patrol { around: PositionRef, radius: f32 },
    Guard { target: TargetRef, radius: f32 },
    Follow { target: TargetRef, radius: f32 },
    Regroup { position: PositionRef, radius: f32 },
    Kite { range: f32 },
    Chase { target: TargetRef },
    Attack { target: TargetRef },
    Flee { from: TargetRef, distance: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetRef {
    NearestEnemy,
    HighestHpEnemy,
    NearestAlly,
    Leader,
    SpawnPoint,
    SquadCenter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PositionRef {
    SpawnPoint,
    SquadCenter,
    Leader,
    SelfCore,
}

/// The single runtime behavior enum. It replaces the old `BehaviorX` marker
/// components and `AIShipController`.
#[derive(Component, Debug, Clone, PartialEq)]
pub enum AiBehavior {
    Idle,
    Wander { point: Vec2 },
    Patrol { around: Vec2, radius: f32 },
    Guard { target: Option<Entity>, radius: f32 },
    Follow { target: Option<Entity>, radius: f32 },
    Regroup { position: Vec2, radius: f32 },
    Kite { target: Option<Entity>, range: f32 },
    Chase { target: Option<Entity> },
    Attack { target: Option<Entity> },
    Flee { from: Option<Entity>, distance: f32 },
}

impl AiBehavior {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Idle => "Idle",
            Self::Wander { .. } => "Wander",
            Self::Patrol { .. } => "Patrol",
            Self::Guard { .. } => "Guard",
            Self::Follow { .. } => "Follow",
            Self::Regroup { .. } => "Regroup",
            Self::Kite { .. } => "Kiting",
            Self::Chase { .. } => "Chase",
            Self::Attack { .. } => "Attack",
            Self::Flee { .. } => "Flee",
        }
    }
}

/// Mutable AI state for one ship.
#[derive(Component, Debug, Clone)]
pub struct AiRuntime {
    pub current: AiBehavior,
    pub spawn_position: Vec2,
    pub rule_cooldowns: Vec<f32>,
    pub think_timer: Timer,
}

impl AiRuntime {
    pub fn new(profile: &AiProfile, spawn_position: Vec2) -> Self {
        Self {
            current: behavior_from_spec(&profile.initial, spawn_position, spawn_position, None),
            spawn_position,
            rule_cooldowns: vec![0.0; profile.rules.len()],
            think_timer: Timer::from_seconds(
                1.0 / AI_THINK_RATE_HZ.max(f32::EPSILON),
                TimerMode::Repeating,
            ),
        }
    }
}

/// Resolve an authorable behavior spec into its runtime form. Entity targets
/// are resolved by the rule evaluator before this is called.
pub fn behavior_from_spec(
    spec: &BehaviorSpec,
    spawn_position: Vec2,
    position: Vec2,
    target: Option<Entity>,
) -> AiBehavior {
    match spec {
        BehaviorSpec::Idle => AiBehavior::Idle,
        BehaviorSpec::Wander { radius } => AiBehavior::Wander {
            point: spawn_position,
        }
        .with_radius(*radius),
        BehaviorSpec::Patrol { radius, .. } => AiBehavior::Patrol {
            around: position,
            radius: *radius,
        },
        BehaviorSpec::Guard { radius, .. } => AiBehavior::Guard {
            target,
            radius: *radius,
        },
        BehaviorSpec::Follow { radius, .. } => AiBehavior::Follow {
            target,
            radius: *radius,
        },
        BehaviorSpec::Regroup { radius, .. } => AiBehavior::Regroup {
            position,
            radius: *radius,
        },
        BehaviorSpec::Kite { range } => AiBehavior::Kite {
            target,
            range: *range,
        },
        BehaviorSpec::Chase { .. } => AiBehavior::Chase { target },
        BehaviorSpec::Attack { .. } => AiBehavior::Attack { target },
        BehaviorSpec::Flee { distance, .. } => AiBehavior::Flee {
            from: target,
            distance: *distance,
        },
    }
}

impl AiBehavior {
    fn with_radius(self, _radius: f32) -> Self {
        // Wander stores only its current point; radius is applied by the
        // evaluator when re-rolling.
        self
    }
}

/// Named presets copied into an inline profile by the editor.
pub const PRESETS: &[&str] = &["Aggressive", "Guard", "Escort", "Skirmisher"];

pub fn preset(name: &str) -> AiProfile {
    let (initial, rules) = match name {
        "Aggressive" => (
            BehaviorSpec::Chase {
                target: TargetRef::NearestEnemy,
            },
            vec![AiRule {
                priority: 10,
                when: AiCondition::HpBelow(0.25),
                then: BehaviorSpec::Flee {
                    from: TargetRef::NearestEnemy,
                    distance: 1000.0,
                },
                cooldown_secs: 8.0,
            }],
        ),
        "Guard" => (
            BehaviorSpec::Guard {
                target: TargetRef::SpawnPoint,
                radius: 400.0,
            },
            vec![AiRule {
                priority: 10,
                when: AiCondition::EnemyInRange(800.0),
                then: BehaviorSpec::Attack {
                    target: TargetRef::NearestEnemy,
                },
                cooldown_secs: 0.0,
            }],
        ),
        "Escort" => (
            BehaviorSpec::Follow {
                target: TargetRef::NearestAlly,
                radius: 300.0,
            },
            vec![AiRule {
                priority: 10,
                when: AiCondition::TargetLost,
                then: BehaviorSpec::Regroup {
                    position: PositionRef::SquadCenter,
                    radius: 300.0,
                },
                cooldown_secs: 2.0,
            }],
        ),
        "Skirmisher" => (
            BehaviorSpec::Wander { radius: 600.0 },
            vec![AiRule {
                priority: 10,
                when: AiCondition::EnemyInRange(1500.0),
                then: BehaviorSpec::Kite { range: 900.0 },
                cooldown_secs: 0.0,
            }],
        ),
        _ => (BehaviorSpec::Idle, vec![]),
    };

    AiProfile { initial, rules }
}

/// Validate an authored profile. Returns a human-readable error list.
pub fn validate_profile(profile: &AiProfile) -> Vec<String> {
    let mut errors = Vec::new();
    for (index, rule) in profile.rules.iter().enumerate() {
        if !rule.cooldown_secs.is_finite() || rule.cooldown_secs < 0.0 {
            errors.push(format!("rule {index}: cooldown must be non-negative"));
        }
    }
    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_round_trips_through_ron() {
        let profile = AiProfile {
            initial: BehaviorSpec::Wander { radius: 500.0 },
            rules: vec![AiRule {
                priority: 10,
                when: AiCondition::HpBelow(0.3),
                then: BehaviorSpec::Flee {
                    from: TargetRef::NearestEnemy,
                    distance: 1000.0,
                },
                cooldown_secs: 5.0,
            }],
        };
        let text = ron::ser::to_string_pretty(&profile, ron::ser::PrettyConfig::default()).unwrap();
        let back: AiProfile = ron::from_str(&text).unwrap();
        assert_eq!(back, profile);
    }

    #[test]
    fn all_presets_validate() {
        for name in PRESETS {
            let profile = preset(name);
            assert!(
                validate_profile(&profile).is_empty(),
                "preset {name} is invalid"
            );
        }
    }

    #[test]
    fn starter_profiles_parse() {
        for text in [
            include_str!("../../../../assets/config/starter/interceptor.ron"),
            include_str!("../../../../assets/config/starter/raider.ron"),
        ] {
            let design: crate::content::ship::ShipDesign = ron::from_str(text).unwrap();
            assert!(!design.ai.rules.is_empty());
        }
    }
}
