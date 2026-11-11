use bevy::prelude::*;
use common::{EnemyKind, UpgradeKind};

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

#[derive(Resource, Default, Debug, Clone)]
pub struct RunStats {
    pub kills: u32,
    pub elapsed_secs: f32,
    pub run_scrap: u32,
    pub extraction_progress_secs: f32,
    pub extraction_hold_secs: f32,
    pub in_extraction_zone: bool,
}

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct ScrapBank {
    pub banked: u32,
}

#[derive(Resource, Default, Debug, Clone, Copy)]
pub enum LastRunOutcome {
    #[default]
    None,
    Extracted {
        scrap_banked: u32,
        kills: u32,
        elapsed_secs: f32,
    },
    Died {
        scrap_lost: u32,
        kills: u32,
        elapsed_secs: f32,
    },
}

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct UpgradeLevels {
    pub weapon_damage: u32,
    pub max_hp: u32,
    pub move_and_reload: u32,
}

impl UpgradeLevels {
    pub fn level(&self, kind: UpgradeKind) -> u32 {
        match kind {
            UpgradeKind::WeaponDamage => self.weapon_damage,
            UpgradeKind::MaxHp => self.max_hp,
            UpgradeKind::MoveAndReloadSpeed => self.move_and_reload,
        }
    }

    pub fn increment(&mut self, kind: UpgradeKind) {
        match kind {
            UpgradeKind::WeaponDamage => self.weapon_damage += 1,
            UpgradeKind::MaxHp => self.max_hp += 1,
            UpgradeKind::MoveAndReloadSpeed => self.move_and_reload += 1,
        }
    }
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct UpgradeModifiers {
    pub damage_multiplier: f32,
    pub bonus_max_hp: f32,
    pub move_speed_multiplier: f32,
    pub reload_speed_multiplier: f32,
}

impl Default for UpgradeModifiers {
    fn default() -> Self {
        Self {
            damage_multiplier: 1.0,
            bonus_max_hp: 0.0,
            move_speed_multiplier: 1.0,
            reload_speed_multiplier: 1.0,
        }
    }
}

pub struct MessagePlugin;

impl Plugin for MessagePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RunStats>()
            .init_resource::<ScrapBank>()
            .init_resource::<LastRunOutcome>()
            .init_resource::<UpgradeLevels>()
            .init_resource::<UpgradeModifiers>()
            .add_message::<DamageMessage>()
            .add_message::<EnemyKilledMessage>()
            .add_message::<WeaponFiredMessage>()
            .add_message::<PlayerDiedMessage>()
            .add_message::<ReloadStartedMessage>()
            .add_message::<ReloadFinishedMessage>()
            .add_message::<ReloadCancelledMessage>()
            .add_message::<WeaponSwappedMessage>()
            .add_message::<ScrapCollectedMessage>()
            .add_message::<ExtractionCompletedMessage>()
            .add_message::<PurchaseUpgradeMessage>()
            .add_message::<UpgradesChangedMessage>()
            .add_message::<DeployRequestedMessage>()
            .add_message::<OpenUpgradesRequestedMessage>();
    }
}
