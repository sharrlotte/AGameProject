use bevy::prelude::*;
use serde::Deserialize;

use crate::content::block::feature::shield::{Shield, ShieldShape};
use crate::validate::{Validate, check};

#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct ShieldConfig {
    pub shape: ShieldShapeConfig,
    pub max_hp: f32,
    pub cooldown: f32,
    pub regen_rate: f32,
    #[serde(default)]
    pub demand: Option<f32>,
}

#[derive(Deserialize, Clone, PartialEq, Debug)]
pub enum ShieldShapeConfig {
    Circle {
        radius: f32,
    },
    Plane {
        width: f32,
        thickness: f32,
        #[serde(default)]
        offset: Vec2,
    },
}

impl Validate for ShieldConfig {
    type Context = ();
    type Error = String;

    fn validate(&self, _: &()) -> Result<(), String> {
        check::positive("shield maximum hit points", self.max_hp)?;
        check::non_negative("shield cooldown duration", self.cooldown)?;
        check::positive("shield regeneration rate", self.regen_rate)?;
        if let Some(demand) = self.demand {
            check::non_negative("shield energy demand", demand)?;
        }
        match self.shape {
            ShieldShapeConfig::Circle { radius } => {
                check::positive("shield circle radius", radius)?;
            }
            ShieldShapeConfig::Plane {
                width,
                thickness,
                offset,
            } => {
                check::positive("shield plane width", width)?;
                check::positive("shield plane thickness", thickness)?;
                check::finite("shield plane horizontal offset", offset.x)?;
                check::finite("shield plane vertical offset", offset.y)?;
            }
        }
        Ok(())
    }
}

impl ShieldConfig {
    pub fn into_shield(self) -> Shield {
        let shape = match self.shape {
            ShieldShapeConfig::Circle { radius } => ShieldShape::Circle { radius },
            ShieldShapeConfig::Plane {
                width,
                thickness,
                offset,
            } => ShieldShape::Plane {
                width,
                thickness,
                offset,
            },
        };
        Shield::new(
            shape,
            self.max_hp,
            self.cooldown,
            self.regen_rate,
            self.demand,
        )
    }
}
