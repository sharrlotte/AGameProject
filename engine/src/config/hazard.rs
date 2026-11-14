use bevy::prelude::*;
use crate::config::RonConfig;
use serde::Deserialize;

/// Global bounds and cadence for ground hazard zones.
#[derive(Asset, Resource, TypePath, Debug, Clone, Deserialize)]
pub struct HazardConfig {
    pub max_active: usize,
    pub tick_interval: f32,
}

impl HazardConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.max_active < 1 {
            return Err("hazard max_active must be at least 1".to_string());
        }
        if !self.tick_interval.is_finite() || self.tick_interval <= 0.0 {
            return Err("hazard tick_interval must be finite and positive".to_string());
        }
        Ok(())
    }
}

impl Default for HazardConfig {
    fn default() -> Self {
        Self {
            max_active: 12,
            tick_interval: 0.5,
        }
    }
}

impl RonConfig for HazardConfig {
    const PATH: &'static str = "config/hazard/hazard.ron";

    fn validate(&self) -> Result<(), String> {
        HazardConfig::validate(self)
    }
}

#[cfg(test)]
mod tests {
use super::*;

    #[test]
    fn hazard_ron_parses_and_validates() {
        let hazard: HazardConfig = crate::assert_ron_config_parses!(HazardConfig);
        hazard.validate().expect("authored hazard config is valid");
        assert!(hazard.max_active >= 1);
        assert!(hazard.tick_interval > 0.0);
    }

    #[test]
    fn invalid_tick_interval_is_rejected() {
        let mut hazard = HazardConfig { max_active: 4, tick_interval: 0.0 };
        assert!(hazard.validate().is_err());

        hazard.tick_interval = f32::NAN;
        assert!(hazard.validate().is_err());
    }

    #[test]
    fn zero_capacity_is_rejected() {
        let hazard = HazardConfig { max_active: 0, tick_interval: 0.5 };
        assert!(hazard.validate().is_err());
    }
}
