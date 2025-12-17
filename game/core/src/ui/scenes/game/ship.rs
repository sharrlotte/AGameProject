use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::content::structures::registry::BlockRegistry;
use crate::{AppState, AppVar};

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Game), spawn_player_ship)
            .add_systems(
                Update,
                ship_movement_controls.run_if(in_state(AppState::Game)),
            );
    }
}

#[derive(Component)]
pub struct PlayerShip;

#[derive(Component)]
pub struct ShipBlockHitbox;

// Define collision groups
pub const GROUP_HULL: Group = Group::GROUP_1;
pub const GROUP_HITBOX: Group = Group::GROUP_2;

pub fn spawn_player_ship(
    mut commands: Commands,
    app_var: Res<AppVar>,
    registry: Res<BlockRegistry>,
    asset_server: Res<AssetServer>,
) {
    let ship_data = if let Some(ship) = &app_var.current_ship {
        ship
    } else {
        warn!("No ship loaded in AppVar!");
        return;
    };

    info!("Spawning ship: {}", ship_data.name);

    let grid_size = 40.0;
    let mut compound_shapes = Vec::new();

    // Spawn the parent entity first so we can attach children
    let ship_entity = commands
        .spawn((
            PlayerShip,
            Transform::from_translation(Vec3::ZERO),
            Visibility::Visible,
            // Physics components
            RigidBody::Dynamic,
            Velocity::zero(),
            ExternalForce::default(),
            ExternalImpulse::default(),
            ReadMassProperties::default(),
            Damping {
                linear_damping: 0.5,  // Low friction for space feel
                angular_damping: 4.0, // Higher damping to prevent spin out
            },
            Restitution::coefficient(0.8), // Bounciness
            ColliderMassProperties::Density(1.0),
            Ccd::enabled(), // Continuous Collision Detection for high speeds
        ))
        .id();

    for saved_block in &ship_data.blocks {
        if let Some(def) = registry.get(&saved_block.block_type) {
            let size_vec = def.shape.size.as_vec2();
            let center_offset = (size_vec - 1.0) * grid_size * 0.5;

            let anchor_pos = Vec2::new(
                saved_block.pos.x as f32 * grid_size,
                saved_block.pos.y as f32 * grid_size,
            );

            let block_pos = anchor_pos + center_offset;

            // Rotation
            let rotation_angle = saved_block.rotation as f32 * std::f32::consts::FRAC_PI_2;

            // Add to compound collider for the HULL (Physics)
            let half_extents = size_vec * grid_size * 0.5;

            // Note: The position in compound collider is relative to the rigid body origin (0,0 of grid).
            compound_shapes.push((
                Vect::new(block_pos.x, block_pos.y),
                rotation_angle,
                Collider::cuboid(half_extents.x, half_extents.y),
            ));

            let base = def.render(&mut commands, &asset_server, saved_block.pos);

            commands.entity(ship_entity).add_child(base);

            // Spawn visual child
            commands.entity(base).with_children(|block_parent| {
                // Spawn Hitbox Collider (Sensor)
                block_parent.spawn((
                    ShipBlockHitbox,
                    Collider::cuboid(half_extents.x, half_extents.y),
                    Sensor,
                    CollisionGroups::new(GROUP_HITBOX, Group::ALL),
                    Transform::default(),
                ));
            });
        }
    }

    // Attach the main hull collider to the ship entity
    if !compound_shapes.is_empty() {
        commands.entity(ship_entity).insert((
            Collider::compound(compound_shapes),
            CollisionGroups::new(GROUP_HULL, Group::ALL),
        ));
    } else {
        warn!("Ship has no blocks! Spawning dummy collider.");
        commands.entity(ship_entity).insert((
            Collider::ball(20.0),
            CollisionGroups::new(GROUP_HULL, Group::ALL),
        ));
    }
}

pub fn ship_movement_controls(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut ExternalForce, &Transform), With<PlayerShip>>,
) {
    let thrust_power = 200000.0; // Needs tuning based on mass
    let turn_speed = 10000000.0; // Needs tuning

    for (mut ext_force, transform) in query.iter_mut() {
        let mut linear_force = Vec2::ZERO;
        let mut torque = 0.0;

        let forward = transform.local_y().truncate(); // Assuming +Y is forward
        let right = transform.local_x().truncate();

        if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
            linear_force += forward * thrust_power;
        }
        if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
            linear_force -= forward * thrust_power;
        }
        if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
            torque += turn_speed;
        }
        if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
            torque -= turn_speed;
        }

        // Strafe
        if keyboard_input.pressed(KeyCode::KeyQ) {
            linear_force -= right * thrust_power;
        }
        if keyboard_input.pressed(KeyCode::KeyE) {
            linear_force += right * thrust_power;
        }

        ext_force.force = linear_force;
        ext_force.torque = torque;
    }
}
