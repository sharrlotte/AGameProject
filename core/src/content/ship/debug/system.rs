use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::content::block::feature::turret::{TurretBarrel, TurretState};
use crate::content::block::BlockOwner;
use crate::content::ship::ai::{Capability, Steering, steering_gizmo_vectors};
use crate::content::ship::{BoundingBox, InspectedShip, Ship, is_inspected, ship_center};

pub fn draw_ship_debug_gizmos(
    mut gizmos: Gizmos,
    inspected: Res<InspectedShip>,
    ships: Query<(
        &Ship,
        &Transform,
        &BoundingBox,
        Option<&Steering>,
        Option<&Capability>,
        &Velocity,
        &ExternalForce,
        &Visibility,
    )>,
    turrets: Query<(&TurretState, &TurretBarrel, &BlockOwner)>,
    global_transforms: Query<&GlobalTransform>,
) {
    let Some(entity) = inspected.0 else {
        return;
    };

    if let Ok((ship, transform, bounds, steering, cap, velocity, force, visibility)) =
        ships.get(entity)
        && *visibility != Visibility::Hidden
    {
        let center = ship_center(transform, bounds);
        let core = transform
            .transform_point(ship.core_position.extend(0.0))
            .truncate();

        if let Some(steering) = steering {
            let gizmo = steering_gizmo_vectors(steering, transform, bounds);
            gizmos.line_2d(
                gizmo.origin,
                gizmo.origin + gizmo.thrust_world * 120.0,
                Color::srgba(0.0, 1.0, 1.0, 0.7),
            );
            if let Some(desired_heading) = gizmo.desired_heading {
                let dir = Vec2::from_angle(desired_heading + std::f32::consts::FRAC_PI_2);
                gizmos.line_2d(
                    gizmo.origin,
                    gizmo.origin + dir * 140.0,
                    Color::srgba(1.0, 0.6, 0.0, 0.7),
                );
            }
        }
        if let Some(cap) = cap {
            gizmos.circle_2d(
                core,
                cap.max_radar_range,
                Color::srgba(0.3, 0.5, 0.9, 0.15),
            );
        }

        let half = bounds.size / 2.0;
        let corners = [
            Vec2::new(-half.x, -half.y),
            Vec2::new(half.x, -half.y),
            Vec2::new(half.x, half.y),
            Vec2::new(-half.x, half.y),
        ]
        .map(|corner| transform.transform_point(corner.extend(0.0)).truncate());
        gizmos.linestrip_2d(
            [corners[0], corners[1], corners[2], corners[3], corners[0]],
            Color::srgb(0.6, 0.7, 0.9),
        );
        gizmos.circle_2d(core, 8.0, Color::srgb(1.0, 1.0, 1.0));

        let heading = transform.local_y().truncate();
        gizmos.line_2d(center, center + heading * 80.0, Color::srgb(0.3, 0.7, 1.0));
        gizmos.line_2d(center, center + velocity.linear, Color::srgb(0.0, 1.0, 0.4));
        gizmos.line_2d(center, center + force.force * 0.01, Color::srgb(1.0, 0.4, 0.2));
    }

    for (state, barrel, owner) in &turrets {
        if !is_inspected(&inspected, owner.body) {
            continue;
        }

        let Ok(barrel_transform) = global_transforms.get(barrel.0) else {
            continue;
        };

        let origin = barrel_transform.translation().truncate();
        gizmos.line_2d(origin, state.aim_point, Color::srgba(1.0, 0.8, 0.0, 0.8));
        gizmos.circle_2d(state.aim_point, 12.0, Color::srgb(1.0, 0.8, 0.0));
    }
}
