use bevy::prelude::*;
use crate::common::{DamageKind, EnemyKind};
use crate::config::RonConfig;
use serde::Deserialize;

/// Runtime stats and composition for one enemy archetype.
#[derive(Debug, Clone, Deserialize)]
pub struct EnemyArchetypeData {
    pub kind: EnemyKind,
    pub max_hp: f32,
    pub move_speed: f32,
    pub radius: f32,
    pub half_height: f32,
    pub mass: f32,
    pub scrap_value: u32,
    pub spawn: SpawnPolicy,
    pub behaviors: Vec<Behavior>,
}

impl EnemyArchetypeData {
    pub fn collider_center_y(&self) -> f32 {
        self.radius + self.half_height
    }

    pub fn melee(&self) -> Option<MeleeSpec> {
        self.behaviors.iter().find_map(Behavior::melee)
    }

    pub fn armor(&self) -> Option<ArmorSpec> {
        self.behaviors.iter().find_map(Behavior::armor)
    }
}

/// Tier gating and selection weight for one archetype.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct SpawnPolicy {
    pub unlock_secs: f32,
    pub weight: f32,
    pub max_active: usize,
}

/// Composable combat capability attached to a spawned archetype.
#[derive(Debug, Clone, Deserialize)]
pub enum Behavior {
    Melee {
        damage: f32,
        cooldown: f32,
        windup: f32,
        reach: f32,
        knockback: f32,
    },
    Armor {
        front_mult: f32,
        back_mult: f32,
        arc_deg: f32,
        #[serde(default)]
        bypass: Vec<DamageKind>,
    },
}

impl Behavior {
    pub fn melee(&self) -> Option<MeleeSpec> {
        match self {
            Self::Melee {
                damage,
                cooldown,
                windup,
                reach,
                knockback,
            } => Some(MeleeSpec {
                damage: *damage,
                cooldown: *cooldown,
                windup: *windup,
                reach: *reach,
                knockback: *knockback,
            }),
            Self::Armor { .. } => None,
        }
    }

    pub fn armor(&self) -> Option<ArmorSpec> {
        match self {
            Self::Armor {
                front_mult,
                back_mult,
                arc_deg,
                bypass,
            } => Some(ArmorSpec {
                front_mult: *front_mult,
                back_mult: *back_mult,
                arc_deg: *arc_deg,
                bypass: bypass.clone(),
            }),
            Self::Melee { .. } => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MeleeSpec {
    pub damage: f32,
    pub cooldown: f32,
    pub windup: f32,
    pub reach: f32,
    pub knockback: f32,
}

#[derive(Debug, Clone)]
pub struct ArmorSpec {
    pub front_mult: f32,
    pub back_mult: f32,
    pub arc_deg: f32,
    pub bypass: Vec<DamageKind>,
}

#[derive(Asset, Resource, TypePath, Debug, Clone, Deserialize)]
pub struct EnemiesConfig {
    pub initial_spawn_delay: f32,
    pub spawn_interval_start: f32,
    pub spawn_interval_min: f32,
    pub escalation_duration_secs: f32,
    pub max_active_start: usize,
    pub max_active_end: usize,
    pub edge_spawn_inset: f32,
    pub min_spawn_dist_from_survivor: f32,
    pub knockback_decay_rate: f32,
    pub hit_flash_duration: f32,
    pub aggro_radius: f32,
    pub deaggro_radius: f32,
    pub deaggro_secs: f32,
    pub alert_secs: f32,
    pub stagger_threshold: f32,
    pub stagger_secs: f32,
    pub separation_radius: f32,
    pub separation_strength: f32,
    pub archetypes: Vec<EnemyArchetypeData>,
}

impl EnemiesConfig {
    pub fn archetype(&self, kind: EnemyKind) -> Option<&EnemyArchetypeData> {
        self.archetypes.iter().find(|arch| arch.kind == kind)
    }

    /// Rejects a roster that could produce invalid combat behaviour at runtime.
    pub fn validate(&self) -> Result<(), String> {
        if self.archetypes.is_empty() {
            return Err("enemy roster must define at least one archetype".to_string());
        }
        if self.archetype(EnemyKind::Walker).is_none() {
            return Err("enemy roster must include Walker".to_string());
        }

        for (index, arch) in self.archetypes.iter().enumerate() {
            let label = arch.kind.label();
            if self.archetypes[index + 1..]
                .iter()
                .any(|other| other.kind == arch.kind)
            {
                return Err(format!("duplicate enemy archetype '{label}'"));
            }
            for (name, value) in [
                ("max_hp", arch.max_hp),
                ("move_speed", arch.move_speed),
                ("radius", arch.radius),
                ("half_height", arch.half_height),
                ("mass", arch.mass),
            ] {
                if !value.is_finite() || value <= 0.0 {
                    return Err(format!("archetype '{label}' {name} must be positive"));
                }
            }
            if !arch.spawn.unlock_secs.is_finite() || arch.spawn.unlock_secs < 0.0 {
                return Err(format!("archetype '{label}' unlock_secs must be non-negative"));
            }
            if !arch.spawn.weight.is_finite() || arch.spawn.weight < 0.0 {
                return Err(format!("archetype '{label}' spawn weight must be non-negative"));
            }
            if arch.spawn.max_active == 0 {
                return Err(format!("archetype '{label}' max_active must be at least 1"));
            }
            for behavior in &arch.behaviors {
                validate_behavior(label, behavior)?;
            }
        }

        if !self.archetypes.iter().any(|arch| arch.spawn.weight > 0.0) {
            return Err("enemy roster needs at least one positive spawn weight".to_string());
        }

        for (name, value) in [
            ("aggro_radius", self.aggro_radius),
            ("deaggro_radius", self.deaggro_radius),
            ("deaggro_secs", self.deaggro_secs),
            ("alert_secs", self.alert_secs),
            ("stagger_threshold", self.stagger_threshold),
            ("stagger_secs", self.stagger_secs),
            ("separation_radius", self.separation_radius),
            ("separation_strength", self.separation_strength),
        ] {
            if !value.is_finite() || value < 0.0 {
                return Err(format!("enemy {name} must be finite and non-negative"));
            }
        }
        if self.deaggro_radius < self.aggro_radius {
            return Err("enemy deaggro_radius must be at least aggro_radius".to_string());
        }

        Ok(())
    }
}

fn validate_behavior(label: &str, behavior: &Behavior) -> Result<(), String> {
    match behavior {
        Behavior::Melee {
            damage,
            cooldown,
            windup,
            reach,
            knockback,
        } => {
            for (name, value) in [
                ("damage", *damage),
                ("cooldown", *cooldown),
                ("windup", *windup),
                ("reach", *reach),
                ("knockback", *knockback),
            ] {
                if !value.is_finite() || value < 0.0 {
                    return Err(format!(
                        "archetype '{label}' melee {name} must be finite and non-negative"
                    ));
                }
            }
        }
        Behavior::Armor {
            front_mult,
            back_mult,
            arc_deg,
            ..
        } => {
            for (name, value) in [("front_mult", *front_mult), ("back_mult", *back_mult)] {
                if !value.is_finite() || value < 0.0 {
                    return Err(format!(
                        "archetype '{label}' armor {name} must be finite and non-negative"
                    ));
                }
            }
            if !arc_deg.is_finite() || *arc_deg <= 0.0 || *arc_deg > 360.0 {
                return Err(format!(
                    "archetype '{label}' armor arc_deg must be in (0, 360]"
                ));
            }
        }
    }
    Ok(())
}

impl Default for EnemiesConfig {
    fn default() -> Self {
        Self {
            initial_spawn_delay: 0.0,
            spawn_interval_start: 1.0,
            spawn_interval_min: 1.0,
            escalation_duration_secs: 1.0,
            max_active_start: 1,
            max_active_end: 1,
            edge_spawn_inset: 2.0,
            min_spawn_dist_from_survivor: 0.0,
            knockback_decay_rate: 10.0,
            hit_flash_duration: 0.09,
            aggro_radius: 18.0,
            deaggro_radius: 26.0,
            deaggro_secs: 2.5,
            alert_secs: 4.0,
            stagger_threshold: 4.5,
            stagger_secs: 0.35,
            separation_radius: 1.2,
            separation_strength: 2.0,
            archetypes: Vec::new(),
        }
    }
}

impl RonConfig for EnemiesConfig {
    const PATH: &'static str = "config/enemies/enemies.ron";

    fn validate(&self) -> Result<(), String> {
        EnemiesConfig::validate(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn authored() -> EnemiesConfig {
        crate::assert_ron_config_parses!(EnemiesConfig)
    }

    #[test]
    fn enemies_ron_parses_and_validates() {
        let enemies = authored();
        enemies.validate().expect("authored roster is valid");
        assert_eq!(enemies.archetypes.len(), 3);
        assert!(enemies.archetype(EnemyKind::Tank).is_some());
    }

    #[test]
    fn duplicate_archetype_is_rejected() {
        let mut enemies = authored();
        let walker = enemies
            .archetype(EnemyKind::Walker)
            .expect("walker archetype")
            .clone();
        enemies.archetypes.push(walker);
        let error = enemies.validate().expect_err("duplicate must be rejected");
        assert!(error.contains("Walker"), "unexpected error: {error}");
    }

    #[test]
    fn missing_walker_is_rejected() {
        let mut enemies = authored();
        enemies.archetypes.retain(|arch| arch.kind != EnemyKind::Walker);
        let error = enemies.validate().expect_err("walker is required");
        assert!(error.contains("Walker"), "unexpected error: {error}");
    }

    #[test]
    fn invalid_armor_arc_is_rejected() {
        let mut enemies = authored();
        let tank = enemies
            .archetypes
            .iter_mut()
            .find(|arch| arch.kind == EnemyKind::Tank)
            .expect("tank archetype");
        for behavior in &mut tank.behaviors {
            if let Behavior::Armor { arc_deg, .. } = behavior {
                *arc_deg = 0.0;
            }
        }
        let error = enemies.validate().expect_err("arc must be positive");
        assert!(error.contains("arc_deg"), "unexpected error: {error}");
    }
}
