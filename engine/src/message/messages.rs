use bevy::prelude::*;
use crate::common::{DamageKind, EnemyKind, UpgradeKind};

#[derive(Message, Debug, Clone, Copy)]
pub struct DamageMessage {
    pub target: Entity,
    pub amount: f32,
    pub knockback: Vec3,
    pub source: Entity,
    pub kind: DamageKind,
    /// Travel direction of the hit. Zero is treated as frontal by armor.
    pub hit_dir: Vec3,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct EnemyKilledMessage {
    pub entity: Entity,
    pub position: Vec3,
    pub kind: EnemyKind,
    pub scrap: u32,
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

/// Why an active reload ended before the magazine was refilled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReloadCancelReason {
    Swapped,
    Died,
    Interrupted,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct ReloadStartedMessage {
    pub entity: Entity,
    pub duration: f32,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct ReloadFinishedMessage {
    pub entity: Entity,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct ReloadCancelledMessage {
    pub entity: Entity,
    pub reason: ReloadCancelReason,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct WeaponSwappedMessage {
    pub entity: Entity,
    pub from: usize,
    pub to: usize,
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

/// Fired when user requests camera zoom adjustments (e.g. mouse wheel).
#[derive(Message, Debug, Clone, Copy)]
pub struct CameraZoomMessage {
    pub steps: f32,
}

