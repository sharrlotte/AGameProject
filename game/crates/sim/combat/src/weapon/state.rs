use bevy::prelude::*;

use crate::config::{WeaponSpecData, WeaponsConfig};

#[derive(Debug, Clone)]
pub struct WeaponSlotState {
    pub spec: WeaponSpecData,
    pub ammo: u32,
}

impl WeaponSlotState {
    pub fn new(spec: WeaponSpecData) -> Self {
        let ammo = spec.mag_size;
        Self { spec, ammo }
    }
}

/// Holds the survivor's equipped weapons and reload/fire cooldown state.
#[derive(Component, Debug, Clone)]
pub struct WeaponInventory {
    pub slots: Vec<WeaponSlotState>,
    pub active_slot: usize,
    pub cooldown_timer: f32,
    pub reload_timer: Option<f32>,
    pub reload_total: f32,
}

impl WeaponInventory {
    pub fn from_config(cfg: &WeaponsConfig) -> Self {
        let slots = cfg
            .weapons
            .iter()
            .cloned()
            .map(WeaponSlotState::new)
            .collect();
        Self {
            slots,
            active_slot: 0,
            cooldown_timer: 0.0,
            reload_timer: None,
            reload_total: 0.0,
        }
    }

    pub fn active(&self) -> Option<&WeaponSlotState> {
        self.slots.get(self.active_slot)
    }

    pub fn active_mut(&mut self) -> Option<&mut WeaponSlotState> {
        self.slots.get_mut(self.active_slot)
    }

    /// Normalized reload progress in `0..=1`, or `None` when no reload is active.
    pub fn reload_progress(&self) -> Option<f32> {
        let remaining = self.reload_timer?;
        if self.reload_total <= 0.0 {
            return Some(0.0);
        }
        Some((1.0 - remaining / self.reload_total).clamp(0.0, 1.0))
    }
}
