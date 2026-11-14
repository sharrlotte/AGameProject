use bevy::prelude::*;
use crate::common::DamageKind;

/// Authored payload for one ground hazard zone.
#[derive(Debug, Clone, Copy)]
pub struct HazardSpec {
    pub radius: f32,
    pub damage: f32,
    pub lifetime: f32,
    pub kind: DamageKind,
}

impl HazardSpec {
    pub fn validate(&self) -> Result<(), String> {
        if !self.radius.is_finite() || self.radius <= 0.0 {
            return Err("hazard radius must be finite and positive".to_string());
        }
        if !self.damage.is_finite() || self.damage < 0.0 {
            return Err("hazard damage must be finite and non-negative".to_string());
        }
        if !self.lifetime.is_finite() || self.lifetime <= 0.0 {
            return Err("hazard lifetime must be finite and positive".to_string());
        }
        Ok(())
    }
}

/// Active ground hazard zone with its authored spec and tick cadence.
#[derive(Component, Debug, Clone, Copy)]
pub struct HazardZone {
    pub spec: HazardSpec,
    pub age: f32,
    pub tick_timer: f32,
}

impl HazardZone {
    pub fn new(spec: HazardSpec, tick_interval: f32) -> Self {
        Self {
            spec,
            age: 0.0,
            tick_timer: tick_interval,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.age >= self.spec.lifetime
    }
}

/// Ground-marker child that a renderer fills with the zone visual.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct HazardMarker;

/// Requests one hazard zone at a horizontal position.
#[derive(Message, Debug, Clone, Copy)]
pub struct HazardSpawnMessage {
    pub position: Vec3,
    pub spec: HazardSpec,
}

pub fn marker_transform(radius: f32) -> Transform {
    Transform::from_scale(Vec3::new(radius, 1.0, radius))
        .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2))
}
