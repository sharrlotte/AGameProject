use bevy::math::FloatPow;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use std::f32::consts::PI;

use crate::config::GameConfigValue;
use crate::constant::DROP_PICKUP_RADIUS;
use crate::content::block::Loot;
use crate::content::ship::ai::{AiConfigValue, Skill, Steering};
use crate::content::ship::stat::ShipStats;
use crate::content::team::Team;
use crate::resource::ControlledShip;

use super::*;

pub fn update_ship_steering(
    mut ships: Query<(
        Entity,
        &Steering,
        &Skill,
        &Transform,
        &mut ExternalForce,
        &mut Velocity,
        &ShipStats,
        &mut Throttle,
    )>,
    controlled: Res<ControlledShip>,
    ai_cfg: Res<AiConfigValue>,
) {
    for (entity, steering, skill, transform, mut force, mut velocity, stats, mut throttle) in
        &mut ships
    {
        if controlled.controlling == Some(entity) {
            continue;
        }

        let thrust_power = stats.thrust_power;
        let turn_speed = stats.rotate_speed;
        let current_angle = transform.rotation.to_euler(EulerRot::XYZ).2;

        let thrust_len = steering.thrust_world.length();
        let thrust_world = if thrust_len > 1.0 {
            steering.thrust_world / thrust_len
        } else {
            steering.thrust_world
        };

        let desired_angle = steering.desired_heading.unwrap_or_else(|| {
            if thrust_world.length_squared() > 1e-4 {
                thrust_world.y.atan2(thrust_world.x) - PI / 2.0
            } else {
                current_angle
            }
        });
        let angle_diff = wrap_angle(desired_angle - current_angle);
        let (torque, torque_multiplier) =
            compute_aim_torque(angle_diff, turn_speed, &mut velocity);
        force.torque = torque;

        let cap = steering.throttle_cap.clamp(0.0, 1.0);
        let linear = if skill.level() >= ai_cfg.0.unlocks.omnidirectional_thrust_level {
            let applied = thrust_world * cap;
            force.force = applied * thrust_power;
            applied.length()
        } else {
            let forward = transform.local_y().truncate();
            let forward_proj = thrust_world.dot(forward).max(0.0);
            if angle_diff.abs() < PI / 3.0 && forward_proj > 0.0 {
                let mag = (forward_proj * cap).clamp(0.0, 1.0);
                force.force = forward * thrust_power * mag;
                mag
            } else {
                force.force = Vec2::ZERO;
                0.0
            }
        };

        let angular = if torque.abs() > 0.0 {
            torque_multiplier
        } else {
            0.0
        };
        throttle.set(linear, angular);
    }
}

fn compute_aim_torque(angle_diff: f32, turn_speed: f32, velocity: &mut Velocity) -> (f32, f32) {
    let torque_multiplier = (angle_diff.abs() / velocity.angular.abs().max(1e-4))
        .clamp(0.0, 1.0)
        .squared();
    let mut torque = if angle_diff > 0.0 {
        turn_speed
    } else {
        -turn_speed
    } * torque_multiplier;

    if angle_diff.abs() < 0.01 && velocity.angular.abs() < 0.01 {
        torque = 0.0;
        velocity.angular = 0.0;
    }
    (torque, torque_multiplier)
}

fn wrap_angle(mut a: f32) -> f32 {
    while a > PI {
        a -= 2.0 * PI;
    }
    while a < -PI {
        a += 2.0 * PI;
    }
    a
}

pub fn collect_drop(
    mut commands: Commands,
    mut drops: Query<(Entity, &mut Transform, &Loot)>,
    mut ships: Query<(Entity, &mut ShipStats, &GlobalTransform, &Team)>,
    time: Res<Time>,
    config: Res<GameConfigValue>,
) {
    for (drop_entity, mut drop_transform, drop_component) in &mut drops {
        let drop_position = drop_transform.translation.truncate();
        let Loot::Item(item) = drop_component;

        let nearest = ships
            .iter_mut()
            .filter(|(_, stats, _, team)| team.is_player() && !stats.is_full())
            .map(|(entity, stats, transform, _)| {
                (entity, stats, transform.translation().truncate())
            })
            .filter(|(_, _, position)| {
                position.distance(drop_position) <= config.0.drop_collect_radius
            })
            .min_by(|a, b| {
                a.2.distance_squared(drop_position)
                    .total_cmp(&b.2.distance_squared(drop_position))
            });

        let Some((_, mut stats, ship_position)) = nearest else {
            continue;
        };

        let direction = ship_position - drop_position;
        let distance = direction.length();
        drop_transform.translation += (config.0.drop_collect_radius
            * direction.normalize_or_zero()
            * time.delta_secs()
            * 3.0)
            .extend(0.0);

        if distance > DROP_PICKUP_RADIUS {
            continue;
        }

        if stats.cargo_used().saturating_add(item.amount) > stats.cargo_capacity {
            continue;
        }
        stats.add_cargo(&item.item_type, item.amount);
        commands.entity(drop_entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GameConfig;
    use crate::content::block::ItemDrop;
    use crate::control::system::toggle_control_mode;
    use crate::control::ControlPlugin;
    use crate::resource::{CameraTarget, ControlMode};

    fn app_with_player_ship() -> (App, Entity) {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<ControlMode>()
            .init_resource::<CameraTarget>()
            .init_resource::<ControlledShip>()
            .init_resource::<ButtonInput<KeyCode>>()
            .add_systems(Update, toggle_control_mode);

        let ship = app.world_mut().spawn((Ship::default(), Team::new(0))).id();
        (app, ship)
    }

    fn app_with_cargo(capacity: u32) -> (App, Entity) {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ControlPlugin))
            .add_systems(Update, collect_drop)
            .insert_resource(GameConfigValue(GameConfig::default()));
        let ship = app
            .world_mut()
            .spawn((
                ShipStats {
                    cargo_capacity: capacity,
                    ..default()
                },
                GlobalTransform::default(),
                Transform::default(),
                Team::new(0),
            ))
            .id();
        (app, ship)
    }

    #[test]
    fn pickup_is_atomic() {
        let (mut app, ship) = app_with_cargo(5);
        app.world_mut().spawn((
            Transform::default(),
            Loot::Item(ItemDrop {
                item_type: "steel".to_string(),
                amount: 6,
            }),
        ));
        app.update();

        let stats = app.world().get::<ShipStats>(ship).unwrap();
        assert_eq!(stats.cargo_used(), 0);
        assert!(stats.cargo_items.is_empty());
    }

    #[test]
    fn pickup_fills_shared_cargo() {
        let (mut app, ship) = app_with_cargo(10);
        app.world_mut().spawn((
            Transform::default(),
            Loot::Item(ItemDrop {
                item_type: "steel".to_string(),
                amount: 4,
            }),
        ));
        app.update();

        let stats = app.world().get::<ShipStats>(ship).unwrap();
        assert_eq!(stats.cargo_used(), 4);
        assert_eq!(stats.cargo_items.get("steel"), Some(&4));
    }

    #[test]
    fn enemy_ships_do_not_collect() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, ControlPlugin))
            .add_systems(Update, collect_drop)
            .insert_resource(GameConfigValue(GameConfig::default()));
        let ship = app
            .world_mut()
            .spawn((
                ShipStats {
                    cargo_capacity: 100,
                    ..default()
                },
                GlobalTransform::default(),
                Transform::default(),
                Team::new(1),
            ))
            .id();
        app.world_mut().spawn((
            Transform::default(),
            Loot::Item(ItemDrop {
                item_type: "steel".to_string(),
                amount: 4,
            }),
        ));
        app.update();

        let stats = app.world().get::<ShipStats>(ship).unwrap();
        assert_eq!(stats.cargo_used(), 0);
    }

    #[test]
    fn f_toggles_pilot_and_free_look() {
        let (mut app, ship) = app_with_player_ship();

        assert_eq!(
            *app.world().resource::<ControlMode>(),
            ControlMode::FreeLook
        );

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyF);
        app.update();

        assert_eq!(*app.world().resource::<ControlMode>(), ControlMode::Pilot);
        assert_eq!(
            app.world().resource::<ControlledShip>().controlling,
            Some(ship)
        );
        assert_eq!(app.world().resource::<CameraTarget>().following, Some(ship));

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyF);
        app.update();

        assert_eq!(
            *app.world().resource::<ControlMode>(),
            ControlMode::FreeLook
        );
        assert_eq!(app.world().resource::<ControlledShip>().controlling, None);
        assert_eq!(app.world().resource::<CameraTarget>().following, None);
    }

    #[test]
    fn aim_at_rot90_offset_aligns_mounted_gun() {
        use crate::content::ship::ai::desired_ship_angle;
        // Target is to the right (+X), so atan2(0, 1) = 0.
        // Forward-mounted (R0, offset 0): ship Z = -PI/2 (nose points +X).
        let r0 = desired_ship_angle(Vec2::X, 0.0);
        assert!((r0 - (-std::f32::consts::FRAC_PI_2)).abs() < 1e-5);

        // Right-mounted (R270, 3 turns CCW = 3*PI/2): ship Z = -2*PI (nose points +Y, right side points +X).
        let r270 = desired_ship_angle(Vec2::X, 3.0 * std::f32::consts::FRAC_PI_2);
        assert!((wrap_angle(r270)).abs() < 1e-5);
    }
}
