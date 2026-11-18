use bevy::prelude::*;

use super::{BoundingBox, ShipBehavior};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ApproachGeometry {
    pub current: Vec2,
    pub target: Vec2,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GoAroundGeometry {
    pub ring_center: Vec2,
    pub ring_radius: f32,
    pub next_position: Vec2,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BehaviorGeometry {
    Approach(ApproachGeometry),
    GoAround(GoAroundGeometry),
    AimAt { current: Vec2, target: Vec2 },
}

pub fn ship_center(transform: &Transform, bounds: &BoundingBox) -> Vec2 {
    transform
        .transform_point((bounds.size / 2.0).extend(0.0))
        .truncate()
}

pub fn approach_geometry(current: Vec2, target: Vec2) -> ApproachGeometry {
    ApproachGeometry { current, target }
}

pub fn go_around_geometry(current: Vec2, center: Vec2, radius: f32) -> GoAroundGeometry {
    let direction = (current - center).normalize_or_zero();
    let position_on_ring = center + direction * radius;
    let next_position = if current.distance(position_on_ring) <= radius {
        position_on_ring + direction.perp() * radius / 2.0
    } else {
        position_on_ring
    };

    GoAroundGeometry {
        ring_center: center,
        ring_radius: radius,
        next_position,
    }
}

pub fn behavior_geometry(
    behavior: &ShipBehavior,
    transform: &Transform,
    bounds: &BoundingBox,
) -> BehaviorGeometry {
    let current = ship_center(transform, bounds);

    match behavior {
        ShipBehavior::Approach { position, .. } => {
            BehaviorGeometry::Approach(approach_geometry(current, *position))
        }
        ShipBehavior::GoAround { position, radius } => {
            BehaviorGeometry::GoAround(go_around_geometry(current, *position, *radius))
        }
        ShipBehavior::AimAt { target, .. } => BehaviorGeometry::AimAt {
            current,
            target: *target,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Vec2, b: Vec2) -> bool {
        (a - b).length() < 1e-5
    }

    #[test]
    fn ship_center_uses_rotated_half_bounds() {
        let bounds = BoundingBox {
            size: Vec2::new(100.0, 50.0),
        };
        let transform = Transform::from_xyz(10.0, 20.0, 0.0);

        assert!(close(
            ship_center(&transform, &bounds),
            Vec2::new(60.0, 45.0)
        ));
    }

    #[test]
    fn go_around_advances_along_ring_when_in_range() {
        let geometry = go_around_geometry(Vec2::new(100.0, 0.0), Vec2::ZERO, 50.0);

        assert!(close(geometry.ring_center, Vec2::ZERO));
        assert_eq!(geometry.ring_radius, 50.0);
        assert!(close(geometry.next_position, Vec2::new(50.0, 25.0)));
    }

    #[test]
    fn go_around_targets_ring_when_out_of_range() {
        let geometry = go_around_geometry(Vec2::new(200.0, 0.0), Vec2::ZERO, 50.0);

        assert!(close(geometry.next_position, Vec2::new(50.0, 0.0)));
    }

    #[test]
    fn behavior_geometry_preserves_behavior_target() {
        let bounds = BoundingBox {
            size: Vec2::splat(40.0),
        };
        let transform = Transform::from_xyz(5.0, 5.0, 0.0);
        let behavior = ShipBehavior::Approach {
            position: Vec2::new(300.0, -100.0),
            radius: 20.0,
        };

        let geometry = behavior_geometry(&behavior, &transform, &bounds);

        let BehaviorGeometry::Approach(approach) = geometry else {
            panic!("expected approach geometry");
        };
        assert!(close(approach.current, Vec2::new(25.0, 25.0)));
        assert!(close(approach.target, Vec2::new(300.0, -100.0)));
    }
}
