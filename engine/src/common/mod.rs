use bevy::prelude::*;
use bevy_rapier3d::prelude::Group;
use serde::Deserialize;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Loading,
    Playing,
    GameOver,
    Upgrades,
}

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameScene {
    #[cfg(not(feature = "dev"))]
    #[default]
    Arena,
    #[cfg(feature = "dev")]
    Arena,
    #[cfg(feature = "dev")]
    #[default]
    AnimTest,
}

/// System set for pre-physics gameplay simulation in `Update`.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SimUpdateSet;

/// System set for post-physics collision and damage resolution in `PostUpdate`.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SimPostPhysicsSet;

/// Contract system set for damage resolution across simulation crates.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DamageResolutionSet;

/// Contract system set for death effects and dead-entity cleanup.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeathResolutionSet;

pub const GROUP_WORLD: Group = Group::GROUP_1;
pub const GROUP_PLAYER: Group = Group::GROUP_2;
pub const GROUP_ENEMY: Group = Group::GROUP_3;
pub const GROUP_PROJECTILE: Group = Group::GROUP_4;
pub const GROUP_PICKUP: Group = Group::GROUP_5;

/// Marks entities spawned for an active run so transitions cleanly despawn them.
#[derive(Component)]
pub struct RunEntity;

#[derive(Component)]
pub struct Obstacle;

#[derive(Component)]
pub struct Survivor;

#[derive(Component)]
pub struct LocalPlayer;

#[derive(Component, Debug, Clone, Copy)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Stamina {
    pub current: f32,
    pub max: f32,
    pub exhausted_timer: f32,
    pub is_sprinting: bool,
}

impl Stamina {
    pub fn new(max: f32) -> Self {
        Self {
            current: max,
            max,
            exhausted_timer: 0.0,
            is_sprinting: false,
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct MovementStats {
    pub walk_speed: f32,
    pub sprint_speed: f32,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct AimDirection(pub Vec3);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum EnemyKind {
    Walker,
    Runner,
    Tank,
    Spitter,
    Exploder,
}

impl EnemyKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Walker => "Walker",
            Self::Runner => "Runner",
            Self::Tank => "Tank",
            Self::Spitter => "Spitter",
            Self::Exploder => "Exploder",
        }
    }
}

/// How incoming damage interacts with armor and future status effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
pub enum DamageKind {
    #[default]
    Ballistic,
    Explosive,
    Acid,
    Shock,
}

/// Directional damage mitigation owned by armored archetypes.
#[derive(Component, Debug, Clone)]
pub struct Armor {
    pub front_mult: f32,
    pub back_mult: f32,
    pub arc_deg: f32,
    pub bypass: Vec<DamageKind>,
}

impl Armor {
    /// Damage multiplier for a hit travelling along `hit_dir` against `facing`.
    ///
    /// A zero or non-finite direction is treated as frontal so malformed input can never
    /// grant the back-damage bonus.
    pub fn multiplier(&self, facing: Vec3, hit_dir: Vec3, kind: DamageKind) -> f32 {
        if self.bypass.contains(&kind) {
            return 1.0;
        }
        if self.is_frontal(facing, hit_dir) {
            self.front_mult
        } else {
            self.back_mult
        }
    }

    fn is_frontal(&self, facing: Vec3, hit_dir: Vec3) -> bool {
        let forward = Vec3::new(facing.x, 0.0, facing.z);
        let travel = Vec3::new(hit_dir.x, 0.0, hit_dir.z);
        if !forward.is_finite()
            || !travel.is_finite()
            || forward.length_squared() <= 1e-6
            || travel.length_squared() <= 1e-6
        {
            return true;
        }
        let half_arc = (self.arc_deg.clamp(0.0, 360.0) * 0.5).to_radians();
        forward.normalize().dot(-travel.normalize()) >= half_arc.cos()
    }
}

/// Marks an enemy whose health reached zero; death effects run before cleanup despawns it.
#[derive(Component, Debug, Clone, Copy)]
pub struct Dying;

/// Decaying displacement impulse accumulated from damage.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Knockback {
    pub velocity: Vec3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpgradeKind {
    WeaponDamage,
    MaxHp,
    MoveAndReloadSpeed,
}

/// Compatibility umbrella plugin that registers config, common, and message systems/resources.
pub struct CoreDataPlugin;

impl bevy::prelude::Plugin for CoreDataPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        use bevy::prelude::AppExtStates;

        app.init_state::<GameScene>()
            .init_state::<GameState>()
            .add_plugins(crate::message::MessagePlugin)
            .add_plugins(crate::config::ConfigPlugin);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn armor() -> Armor {
        Armor {
            front_mult: 0.3,
            back_mult: 1.0,
            arc_deg: 120.0,
            bypass: vec![DamageKind::Explosive],
        }
    }

    #[test]
    fn damage_kind_defaults_to_ballistic() {
        assert_eq!(DamageKind::default(), DamageKind::Ballistic);
    }

    #[test]
    fn frontal_hit_uses_front_multiplier() {
        let mult = armor().multiplier(Vec3::NEG_Z, Vec3::Z, DamageKind::Ballistic);
        assert!((mult - 0.3).abs() < 1e-6);
    }

    #[test]
    fn rear_hit_uses_back_multiplier() {
        let mult = armor().multiplier(Vec3::NEG_Z, Vec3::NEG_Z, DamageKind::Ballistic);
        assert!((mult - 1.0).abs() < 1e-6);
    }

    #[test]
    fn side_hit_outside_arc_uses_back_multiplier() {
        let mult = armor().multiplier(Vec3::NEG_Z, Vec3::X, DamageKind::Ballistic);
        assert!((mult - 1.0).abs() < 1e-6);
    }

    #[test]
    fn bypass_kind_ignores_armor() {
        let mult = armor().multiplier(Vec3::NEG_Z, Vec3::Z, DamageKind::Explosive);
        assert!((mult - 1.0).abs() < 1e-6);
    }

    #[test]
    fn zero_and_non_finite_directions_are_frontal() {
        for dir in [
            Vec3::ZERO,
            Vec3::new(f32::NAN, 0.0, 0.0),
            Vec3::new(f32::INFINITY, 0.0, 0.0),
        ] {
            let mult = armor().multiplier(Vec3::NEG_Z, dir, DamageKind::Ballistic);
            assert!((mult - 0.3).abs() < 1e-6, "unexpected multiplier for {dir:?}");
        }
    }

    #[test]
    fn missing_facing_is_treated_as_frontal() {
        let mult = armor().multiplier(Vec3::ZERO, Vec3::Z, DamageKind::Ballistic);
        assert!((mult - 0.3).abs() < 1e-6);
    }
}

