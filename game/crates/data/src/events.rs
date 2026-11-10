use bevy::prelude::*;

use crate::state::{EnemyKind, UpgradeKind};

#[derive(Message, Debug, Clone, Copy)]
pub struct DamageMessage {
    pub target: Entity,
    pub amount: f32,
    pub knockback: Vec3,
    pub source: Entity,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct EnemyKilledMessage {
    pub entity: Entity,
    pub position: Vec3,
    pub kind: EnemyKind,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct WeaponFiredMessage {
    pub shooter: Entity,
    pub origin: Vec3,
    pub direction: Vec3,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct PlayerDiedMessage {
    pub survivor: Entity,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct ScrapCollectedMessage {
    pub collector: Entity,
    pub amount: u32,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct ExtractionCompletedMessage {
    pub survivor: Entity,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct PurchaseUpgradeMessage {
    pub kind: UpgradeKind,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct UpgradesChangedMessage;

#[derive(Message, Debug, Clone, Copy)]
pub struct DeployRequestedMessage;

#[derive(Message, Debug, Clone, Copy)]
pub struct OpenUpgradesRequestedMessage;

#[derive(Message, Debug, Clone, Copy)]
pub struct ConfigHotReloadedMessage;
