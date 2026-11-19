use bevy::prelude::*;

use super::BoundingBox;

#[cfg(any(feature = "dev", test))]
pub fn ship_center(transform: &Transform, bounds: &BoundingBox) -> Vec2 {
    transform
        .transform_point(bounds.center.extend(0.0))
        .truncate()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ship_center_uses_rotated_bounds_center() {
        let bounds = BoundingBox::from_min_max(Vec2::new(-20.0, 10.0), Vec2::new(80.0, 60.0));
        let mut transform = Transform::from_xyz(10.0, 20.0, 0.0);
        transform.rotate_z(std::f32::consts::FRAC_PI_2);

        // Local center is (30, 35); rotated +90 deg -> (-35, 30); + (10, 20) -> (-25, 50)
        assert!((ship_center(&transform, &bounds) - Vec2::new(-25.0, 50.0)).length() < 1e-4);
    }
}
