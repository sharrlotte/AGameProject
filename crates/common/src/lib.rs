use bevy::prelude::*;
use bevy_rapier3d::prelude::Group;

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

pub const GROUP_WORLD: Group = Group::GROUP_1;
pub const GROUP_PLAYER: Group = Group::GROUP_2;
pub const GROUP_ENEMY: Group = Group::GROUP_3;
pub const GROUP_PROJECTILE: Group = Group::GROUP_4;
pub const GROUP_PICKUP: Group = Group::GROUP_5;

/// Marks entities spawned for an active run so transitions cleanly despawn them.
#[derive(Component)]
pub struct RunEntity;

/// Shared lightweight markers enabling feature crates to compile in parallel.
#[derive(Component)]
pub struct SurvivorTag;

#[derive(Component)]
pub struct EnemyTag;

#[derive(Component)]
pub struct ObstacleTag;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyKind {
    Walker,
    Runner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpgradeKind {
    WeaponDamage,
    MaxHp,
    MoveAndReloadSpeed,
}
