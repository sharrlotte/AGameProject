use bevy::math::FloatPow;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use std::collections::HashMap;
use std::f32::consts::PI;

use crate::config::GameConfigValue;
use crate::constant::DROP_PICKUP_RADIUS;
use crate::content::block::feature::ammo::{Ammo, AmmoState};
use crate::content::block::stat::StatsUpdateSet;
use crate::content::block::{BlockOwner, Loot};
use crate::content::ship::ai::AiPlugin;
use crate::content::ship::indicator::IndicatorsPlugin;
use crate::content::ship::stat::{ShipChanged, ShipStats, rebuild_cargo_capacity, update_stats};
use crate::content::team::Team;
use crate::resource::{CameraTarget, ControlMode, ControlledShip, SceneState};
use crate::simulation::SimulationSet;

use super::*;

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ShipStats>()
            .register_type::<Throttle>();
        #[cfg(debug_assertions)]
        app.add_systems(
            FixedUpdate,
            update_ship_ui
                .run_if(in_state(SceneState::Game))
                .in_set(SimulationSet::Present),
        );
        app.add_systems(
            PostUpdate,
            update_ship_behavior
                .run_if(in_state(SceneState::Game))
                .in_set(SimulationSet::Present),
        )
        .add_systems(
            FixedPreUpdate,
            update_stats
                .after(StatsUpdateSet)
                .run_if(in_state(SceneState::Game))
                .in_set(SimulationSet::Stats),
        )
        .add_systems(
            FixedPreUpdate,
            rebuild_cargo_capacity
                .run_if(on_message::<ShipChanged>)
                .run_if(in_state(SceneState::Game))
                .in_set(SimulationSet::Stats),
        )
        .add_systems(
            FixedUpdate,
            collect_drop
                .run_if(in_state(SceneState::Game))
                .in_set(SimulationSet::Control),
        )
        .add_plugins((IndicatorsPlugin, AiPlugin, ShipMutationPlugin));
    }
}

#[cfg(debug_assertions)]
pub fn update_ship_ui(
    mut commands: Commands,
    mut ui_roots: Query<(Entity, &ShipUIRoot, &mut Transform)>,
    ships: Query<(
        &Ship,
        &crate::content::ship::ai::AiRuntime,
        Option<&ShipBehavior>,
        &GlobalTransform,
        &ShipStats,
    )>,
    mut blocks: Query<(&BlockOwner, Option<&Ammo>, Option<&AmmoState>)>,
) {
    let mut data: HashMap<Entity, String> = HashMap::new();

    for (owner, ammo, ammo_state) in blocks.iter_mut() {
        let mut string = String::new();

        if let Some(ammo) = ammo {
            string.push_str(format!("\n{ammo:?}").as_str());
        }

        if let Some(ammo_state) = ammo_state {
            string.push_str(format!("\n{ammo_state:?}").as_str());
        }

        if !string.is_empty() {
            data.entry(owner.ship)
                .and_modify(|f| f.push_str(string.as_str()))
                .or_insert(string);
        }
    }

    for (ui_entity, ui_root, _) in ui_roots.iter_mut() {
        if let Ok((ship, ai_runtime, behavior, global_transform, stats)) = ships.get(ui_root.ship) {
            let current = ai_runtime.current.label();
            let empty = String::new();
            let string = data.get(&ui_root.ship).unwrap_or(&empty);

            commands.entity(ui_entity).insert((
                Text2d::new(format!(
                    "Ship: {ship:?}\nC: {current}\nB: {behavior:?}\nCARGO {}/{} {:?}\nS: {stats:?}\n{string}",
                    stats.cargo_used, stats.cargo_capacity, stats.cargo_items,
                )),
                TextFont {
                    font_size: FontSize::Px(30.0),
                    ..default()
                },
                Transform::from_rotation(global_transform.rotation().inverse())
                    .with_translation(Vec2::ZERO.extend(crate::constant::LAYER_SHIP_UI)),
            ));
        }
    }
}
pub fn update_ship_behavior(
    mut ships: Query<(
        Entity,
        &Ship,
        &BoundingBox,
        &ShipBehavior,
        &Transform,
        &mut ExternalForce,
        &mut Velocity,
        &ShipStats,
        &mut Throttle,
    )>,
    controlled: Res<ControlledShip>,
    #[cfg_attr(not(debug_assertions), allow(unused_variables))] mut gizmos: Gizmos,
    config: Res<GameConfigValue>,
) {
    for (entity, _, size, behavior, transform, mut force, mut velocity, stats, mut throttle) in
        ships.iter_mut()
    {
        if let Some(controlling) = controlled.controlling
            && controlling == entity
        {
            continue;
        }

        let thrust_power = stats.thrust_power;
        let turn_speed = stats.rotate_speed;

        let center = size.size / 2.0;

        let current_postion = transform.transform_point(center.extend(0.0)).truncate();
        let forward = transform.local_y().truncate();

        match behavior {
            ShipBehavior::Approach { position, radius } => {
                let direction = (position - current_postion).normalize_or_zero();
                let angle = direction.y.atan2(direction.x) - PI / 2.0;
                let angle_diff = wrap_angle(angle - transform.rotation.to_euler(EulerRot::XYZ).2);
                let distance = current_postion.distance(*position);

                if cfg!(debug_assertions) {
                    gizmos.line_2d(current_postion, *position, Color::srgba(0.0, 1.0, 1.0, 0.5));

                    gizmos.circle_2d(*position, 30.0, Color::srgb(0.0, 1.0, 1.0));
                }

                let force_multiplier = if distance > *radius {
                    1.0
                } else {
                    (distance / radius).squared()
                };

                let torque_multiplier: f32 = (angle_diff.abs() / velocity.angular.abs())
                    .clamp(0.0, 1.0)
                    .squared();

                let mut torque: f32 = if angle_diff > 0.0 {
                    turn_speed
                } else {
                    -turn_speed
                } * torque_multiplier;

                if angle_diff.abs() < 0.01 && velocity.angular.abs() < 0.01 {
                    torque = 0.0;
                    velocity.angular = 0.0;
                }

                force.torque = torque;

                let speed = velocity.linear.length();
                let stop_distance = speed * speed / (2.0 * config.0.linear_damping);

                let linear = if distance <= stop_distance {
                    force.force = Vec2::ZERO;
                    0.0
                } else if angle_diff.abs() < PI / 3.0 && distance > *radius {
                    force.force = forward * thrust_power * force_multiplier;
                    force_multiplier
                } else {
                    force.force = Vec2::ZERO;
                    0.0
                };
                let angular = if torque.abs() > 0.0 {
                    torque_multiplier
                } else {
                    0.0
                };
                throttle.set(linear, angular);
            }
            ShipBehavior::GoAround { position, radius } => {
                let direction = (current_postion - position).normalize_or_zero();
                let position_on_ring = position + direction * radius;
                let mut next_position = position_on_ring;

                let in_range = current_postion.distance(position_on_ring) <= *radius;

                if in_range {
                    next_position = position_on_ring + direction.perp() * radius / 2.0;
                }

                if cfg!(debug_assertions) {
                    gizmos.line_2d(
                        current_postion,
                        next_position,
                        Color::srgba(0.0, 1.0, 0.0, 0.5),
                    );

                    gizmos.circle_2d(*position, *radius, Color::srgb(1.0, 1.0, 0.0));
                    gizmos.circle_2d(next_position, 30.0, Color::srgb(0.0, 1.0, 0.0));
                }

                let direction = (next_position - current_postion).normalize_or_zero();

                let angle = direction.y.atan2(direction.x) - PI / 2.0;
                let angle_diff = wrap_angle(angle - transform.rotation.to_euler(EulerRot::XYZ).2);
                let distance = current_postion.distance(next_position);

                let force_multiplier = if distance > *radius {
                    1.0
                } else {
                    distance / radius
                };

                let torque_multiplier: f32 = (angle_diff.abs() / velocity.angular.abs())
                    .clamp(0.0, 1.0)
                    .squared();

                let mut torque: f32 = if angle_diff > 0.0 {
                    turn_speed
                } else {
                    -turn_speed
                } * torque_multiplier;

                if angle_diff.abs() < 0.01 && velocity.angular.abs() < 0.01 {
                    torque = 0.0;
                    velocity.angular = 0.0;
                }

                force.torque = torque;

                let linear = if angle_diff.abs() < PI / 3.0 {
                    force.force = forward * thrust_power * force_multiplier;
                    force_multiplier
                } else {
                    force.force = Vec2::ZERO;
                    0.0
                };
                let angular = if torque.abs() > 0.0 {
                    torque_multiplier
                } else {
                    0.0
                };
                throttle.set(linear, angular);
            }
        }
    }
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

        if stats.cargo_used.saturating_add(item.amount) > stats.cargo_capacity {
            continue;
        }
        stats.cargo_used += item.amount;
        stats
            .cargo_items
            .entry(item.item_type.clone())
            .and_modify(|count| *count += item.amount)
            .or_insert(item.amount);
        commands.entity(drop_entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app_with_player_ship() -> (App, Entity) {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<ControlMode>()
            .init_resource::<CameraTarget>()
            .init_resource::<ControlledShip>()
            .init_resource::<ButtonInput<KeyCode>>()
            .add_systems(Update, crate::control::system::toggle_control_mode);

        let ship = app.world_mut().spawn((Ship::default(), Team::new(0))).id();
        (app, ship)
    }

    fn app_with_cargo(capacity: u32) -> (App, Entity) {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, crate::control::ControlPlugin))
            .add_systems(Update, collect_drop)
            .insert_resource(GameConfigValue(crate::config::GameConfig::default()));
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
            Loot::Item(crate::content::block::ItemDrop {
                item_type: "steel".to_string(),
                amount: 6,
            }),
        ));
        app.update();

        let stats = app.world().get::<ShipStats>(ship).unwrap();
        assert_eq!(stats.cargo_used, 0);
        assert!(stats.cargo_items.is_empty());
    }

    #[test]
    fn pickup_fills_shared_cargo() {
        let (mut app, ship) = app_with_cargo(10);
        app.world_mut().spawn((
            Transform::default(),
            Loot::Item(crate::content::block::ItemDrop {
                item_type: "steel".to_string(),
                amount: 4,
            }),
        ));
        app.update();

        let stats = app.world().get::<ShipStats>(ship).unwrap();
        assert_eq!(stats.cargo_used, 4);
        assert_eq!(stats.cargo_items.get("steel"), Some(&4));
    }

    #[test]
    fn enemy_ships_do_not_collect() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, crate::control::ControlPlugin))
            .add_systems(Update, collect_drop)
            .insert_resource(GameConfigValue(crate::config::GameConfig::default()));
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
            Loot::Item(crate::content::block::ItemDrop {
                item_type: "steel".to_string(),
                amount: 4,
            }),
        ));
        app.update();

        let stats = app.world().get::<ShipStats>(ship).unwrap();
        assert_eq!(stats.cargo_used, 0);
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
}
