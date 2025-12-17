use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use serde::{Deserialize, Serialize};

use crate::content::{structures::ship::PlayerShip, team::Team};

pub struct ShootingFeaturePlugin;

#[derive(Component)]
pub struct Shooter {
    pub fire_rate: f32,
    pub projectile: String,
    pub last_fire_time: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct ShooterFeature {
    pub fire_rate: f32,
    pub projectile: String,
}

impl Plugin for ShootingFeaturePlugin {
    fn build(&self, app: &mut App) {
        info!("Registering shooting feature");
        app.add_systems(Update, (handle_shooting, despawn_after_lifetime));
    }
}

#[derive(Component)]
pub struct Bullet {
    pub timer: Timer,
}

pub fn handle_shooting(
    time: Res<Time>,
    mut shooter_query: Query<(&GlobalTransform, &mut Shooter, Option<&Team>), With<PlayerShip>>,
    mut commands: Commands,
) {
    for (global_transform, mut shooter, team_opt) in &mut shooter_query {
        // Use the team from the component, or default to Ally if missing
        let team = team_opt.copied().unwrap_or_default();

        // Get current global rotation around Z
        let (_, current_rot, _) = global_transform.to_scale_rotation_translation();

        let direction = current_rot * Vec3::Y;
        let direction2d = direction.truncate();

        if shooter.last_fire_time < time.elapsed_secs() - shooter.fire_rate {
            commands.spawn((
                Bullet {
                    timer: Timer::from_seconds(10.0, TimerMode::Once),
                },
                Sprite {
                    color: Color::srgb(1.0, 0.0, 0.0),
                    ..default()
                },
                Transform::from_translation(global_transform.translation()),
                RigidBody::Dynamic,
                Collider::ball(2.0),
                Sensor, // bullets usually don't push ships
                Velocity::linear(direction2d * 1000.0),
                ActiveEvents::COLLISION_EVENTS,
                GravityScale(0.0),
                CollisionGroups::new(Group::default(), team.get_bullet_filter()),
                team,
            ));

            shooter.last_fire_time = time.elapsed_secs();
        }
    }
}

fn despawn_after_lifetime(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Bullet)>,
) {
    for (entity, mut lifetime) in query.iter_mut() {
        lifetime.timer.tick(time.delta());

        if lifetime.timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}
