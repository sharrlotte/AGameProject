use serde::Deserialize;

use crate::content::block::feature::shield::{Shield, ShieldShape};
use crate::validate::{FiniteVec2, NonNegativeF32, PositiveF32};

#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct ShieldConfig {
    pub shape: ShieldShapeConfig,
    pub max_hp: PositiveF32,
    pub cooldown: NonNegativeF32,
    pub regen_rate: PositiveF32,
    #[serde(default)]
    pub demand: Option<NonNegativeF32>,
}

#[derive(Deserialize, Clone, PartialEq, Debug)]
pub enum ShieldShapeConfig {
    Circle {
        radius: PositiveF32,
    },
    Plane {
        width: PositiveF32,
        thickness: PositiveF32,
        #[serde(default)]
        offset: FiniteVec2,
    },
}

impl ShieldConfig {
    pub fn into_shield(self) -> Shield {
        let shape = match self.shape {
            ShieldShapeConfig::Circle { radius } => ShieldShape::Circle {
                radius: radius.get(),
            },
            ShieldShapeConfig::Plane {
                width,
                thickness,
                offset,
            } => ShieldShape::Plane {
                width: width.get(),
                thickness: thickness.get(),
                offset: offset.get(),
            },
        };
        Shield::new(
            shape,
            self.max_hp.get(),
            self.cooldown.get(),
            self.regen_rate.get(),
            self.demand.map(NonNegativeF32::get),
        )
    }
}
