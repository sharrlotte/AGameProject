pub mod apply;
pub mod capture;
pub mod component;

pub use apply::*;
pub use capture::*;
pub use component::*;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::content::body::Debris;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BodySnapshot {
    Ship(ShipSnapshot),
    Debris(DebrisSnapshot),
}

impl BodySnapshot {
    pub fn capture(world: &World, entity: EntityRef) -> Option<Self> {
        if entity.get::<Debris>().is_some() {
            return DebrisSnapshot::capture(world, entity).map(Self::Debris);
        }

        ShipSnapshot::capture(world, entity).map(Self::Ship)
    }
}
