pub mod apply;
pub mod capture;
pub mod component;

pub use apply::*;
pub use capture::*;
pub use component::*;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BodySnapshot {
    Ship(ShipSnapshot),
    Debris(DebrisSnapshot),
}

impl BodySnapshot {
    pub fn capture(world: &World, entity: EntityRef) -> Option<Self> {
        if entity.get::<crate::content::body::Debris>().is_some() {
            return DebrisSnapshot::capture(world, entity).map(Self::Debris);
        }

        ShipSnapshot::capture(world, entity).map(Self::Ship)
    }
}
