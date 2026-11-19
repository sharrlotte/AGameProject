use bevy::prelude::*;

use super::BoundingBox;

#[cfg(any(feature = "dev", test))]
pub fn ship_center(transform: &Transform, bounds: &BoundingBox) -> Vec2 {
    transform
        .transform_point((bounds.size / 2.0).extend(0.0))
        .truncate()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ship_center_uses_rotated_half_bounds() {
        let bounds = BoundingBox {
            size: Vec2::new(100.0, 50.0),
        };
        let transform = Transform::from_xyz(10.0, 20.0, 0.0);

        assert!((ship_center(&transform, &bounds) - Vec2::new(60.0, 45.0)).length() < 1e-5);
    }
}
