use bevy::prelude::*;
use uuid::Uuid;

use crate::content::ship::stat::ShipStats;
use crate::content::ship::throttle::Throttle;


#[derive(Component, Debug, Default, Clone)]
#[require(Throttle)]
pub struct Ship {
    pub id: Uuid,
    pub core_id: Uuid,
    pub core_position: Vec2,
}


#[derive(Component, Debug, Clone, Default)]
pub struct BoundingBox {
    min: Vec2,
    max: Vec2,
    center: Vec2,
    size: Vec2,
}

impl BoundingBox {
    pub fn from_min_max(min: Vec2, max: Vec2) -> Self {
        Self {
            min,
            max,
            center: (min + max) * 0.5,
            size: max - min,
        }
    }

    pub fn min(&self) -> Vec2 { self.min }
    pub fn max(&self) -> Vec2 { self.max }
    pub fn center(&self) -> Vec2 { self.center }
    pub fn size(&self) -> Vec2 { self.size }

    pub fn corners(&self) -> [Vec2; 4] {
        [
            Vec2::new(self.min.x, self.min.y),
            Vec2::new(self.max.x, self.min.y),
            Vec2::new(self.max.x, self.max.y),
            Vec2::new(self.min.x, self.max.y),
        ]
    }
}


#[derive(Clone, Debug)]
pub struct GridBuildResult {
    pub ship: Ship,
    pub stats: ShipStats,
    pub bounds: BoundingBox,
}


pub use crate::save::design::ShipDesign;


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounding_box_from_min_max_sets_center_size_and_corners() {
        let bounds = BoundingBox::from_min_max(Vec2::new(-16.0, 32.0), Vec2::new(48.0, 96.0));
        assert_eq!(bounds.min, Vec2::new(-16.0, 32.0));
        assert_eq!(bounds.max, Vec2::new(48.0, 96.0));
        assert_eq!(bounds.center(), Vec2::new(16.0, 64.0));
        assert_eq!(bounds.size(), Vec2::new(64.0, 64.0));
        assert_eq!(
            bounds.corners(),
            [
                Vec2::new(-16.0, 32.0),
                Vec2::new(48.0, 32.0),
                Vec2::new(48.0, 96.0),
                Vec2::new(-16.0, 96.0),
            ]
        );
    }
}
