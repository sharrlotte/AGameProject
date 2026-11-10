use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::GameState;

use super::combat::WeaponInventory;
use super::input_intent::ActionIntent;
use super::{DamageMessage, GROUP_ENEMY, GROUP_PLAYER, GROUP_WORLD, RunEntity};

pub const AIM_HEIGHT: f32 = 0.9;
pub const PLAYER_RADIUS: f32 = 0.45;
pub const PLAYER_HALF_HEIGHT: f32 = 0.45;

const WALK_SPEED: f32 = 7.5;
const SPRINT_SPEED: f32 = 11.0;
const MAX_STAMINA: f32 = 100.0;
const STAMINA_DRAIN_PER_SEC: f32 = 28.0;
const STAMINA_REGEN_PER_SEC: f32 = 22.0;
const STAMINA_EXHAUSTION_DELAY: f32 = 1.0;
const PLAYER_MAX_HP: f32 = 100.0;

/// Simulation marker for player-controlled survivors (multiplayer-ready).
#[derive(Component)]
pub struct Survivor;

/// Presentation marker identifying the local client's survivor.
#[derive(Component)]
pub struct LocalPlayer;

#[derive(Component, Debug, Clone, Copy)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct Stamina {
    pub current: f32,
    pub max: f32,
    pub exhausted_timer: f32,
    pub is_sprinting: bool,
}

impl Default for Stamina {
    fn default() -> Self {
        Self {
            current: MAX_STAMINA,
            max: MAX_STAMINA,
            exhausted_timer: 0.0,
            is_sprinting: false,
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
pub struct MovementStats {
    pub walk_speed: f32,
    pub sprint_speed: f32,
}

impl Default for MovementStats {
    fn default() -> Self {
        Self {
            walk_speed: WALK_SPEED,
            sprint_speed: SPRINT_SPEED,
        }
    }
}

/// Unit XZ forward aim vector for the survivor.
#[derive(Component, Debug, Clone, Copy)]
pub struct AimDirection(pub Vec3);

/// Spawns the local survivor at the center of the arena when a run starts.
pub fn spawn_player(mut commands: Commands) {
    commands.spawn((
        (
            Name::new("Survivor"),
            Survivor,
            LocalPlayer,
            RunEntity,
            ActionIntent::default(),
            Health::new(PLAYER_MAX_HP),
            Stamina::default(),
            MovementStats::default(),
            AimDirection(Vec3::NEG_Z),
            WeaponInventory::default(),
        ),
        (
            RigidBody::Dynamic,
            Collider::capsule_y(PLAYER_HALF_HEIGHT, PLAYER_RADIUS),
            LockedAxes::TRANSLATION_LOCKED_Y | LockedAxes::ROTATION_LOCKED,
            Velocity::zero(),
            Sleeping::disabled(),
            TransformInterpolation::default(),
            Damping {
                linear_damping: 0.0,
                angular_damping: 0.0,
            },
            Friction {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Min,
            },
            Restitution {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Min,
            },
            CollisionGroups::new(GROUP_PLAYER, GROUP_WORLD | GROUP_ENEMY),
            Transform::from_xyz(0.0, AIM_HEIGHT, 0.0),
        ),
    ));
}

/// Updates survivor stamina, planar movement velocity, and aim direction (rotation is applied to child visuals).
pub fn player_movement_and_aim(
    time: Res<Time>,
    mut query: Query<
        (
            &ActionIntent,
            &MovementStats,
            &mut Stamina,
            &mut Velocity,
            &Transform,
            &mut AimDirection,
        ),
        With<Survivor>,
    >,
) {
    let dt = time.delta_secs();

    for (intent, stats, mut stamina, mut velocity, transform, mut aim_dir) in &mut query {
        let move_dir = if intent.move_dir.length_squared() > 1e-4 {
            intent.move_dir.normalize()
        } else {
            Vec2::ZERO
        };
        let is_moving = move_dir != Vec2::ZERO;

        let can_sprint = stamina.exhausted_timer <= 0.0 && stamina.current > 0.0;
        let sprinting = intent.sprint && is_moving && can_sprint;
        stamina.is_sprinting = sprinting;

        if sprinting {
            stamina.current = (stamina.current - STAMINA_DRAIN_PER_SEC * dt).max(0.0);
            if stamina.current <= 0.0 {
                stamina.exhausted_timer = STAMINA_EXHAUSTION_DELAY;
            }
        } else {
            if stamina.exhausted_timer > 0.0 {
                stamina.exhausted_timer = (stamina.exhausted_timer - dt).max(0.0);
            }
            if stamina.exhausted_timer <= 0.0 {
                stamina.current = (stamina.current + STAMINA_REGEN_PER_SEC * dt).min(stamina.max);
            }
        }

        let speed = if sprinting {
            stats.sprint_speed
        } else {
            stats.walk_speed
        };
        velocity.linear = Vec3::new(move_dir.x * speed, 0.0, move_dir.y * speed);

        if let Some(aim_world) = intent.aim_world {
            let flat_dir = Vec3::new(
                aim_world.x - transform.translation.x,
                0.0,
                aim_world.z - transform.translation.z,
            );
            if flat_dir.length_squared() > 1e-4 {
                aim_dir.0 = flat_dir.normalize();
            }
        }
    }
}

/// Applies incoming damage to survivors and triggers Game Over at 0 HP.
pub fn apply_player_damage(
    mut damage_reader: MessageReader<DamageMessage>,
    mut survivors: Query<&mut Health, With<Survivor>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let mut any_died = false;
    for msg in damage_reader.read() {
        let Ok(mut health) = survivors.get_mut(msg.target) else {
            continue;
        };
        if health.current <= 0.0 {
            continue;
        }
        health.current = (health.current - msg.amount).max(0.0);
        if health.current <= 0.0 {
            any_died = true;
        }
    }

    if any_died {
        next_state.set(GameState::GameOver);
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use bevy::state::app::StatesPlugin;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_state::<GameState>()
            .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
                Duration::from_secs_f32(1.0 / 60.0),
            ))
            .add_message::<DamageMessage>()
            .add_systems(Startup, spawn_player)
            .add_systems(Update, (player_movement_and_aim, apply_player_damage));
        app
    }

    #[test]
    fn movement_sprint_stamina_and_aim_work() {
        let mut app = test_app();
        app.update();

        let player = app
            .world_mut()
            .query_filtered::<Entity, With<Survivor>>()
            .single(app.world())
            .expect("player spawned");

        {
            let mut intent = app.world_mut().get_mut::<ActionIntent>(player).unwrap();
            intent.move_dir = Vec2::new(1.0, 1.0); // unnormalized diagonal
            intent.sprint = true;
            intent.aim_world = Some(Vec3::new(10.0, AIM_HEIGHT, 0.0));
        }

        for _ in 0..30 {
            app.update();
        }

        let vel = app.world().get::<Velocity>(player).unwrap();
        let speed = Vec2::new(vel.linear.x, vel.linear.z).length();
        assert!((speed - SPRINT_SPEED).abs() < 0.01, "expected sprint speed, got {speed}");

        let stamina = app.world().get::<Stamina>(player).unwrap();
        assert!(stamina.current < MAX_STAMINA);
        assert!(stamina.is_sprinting);

        let aim = app.world().get::<AimDirection>(player).unwrap();
        assert!((aim.0 - Vec3::X).length() < 0.01);
    }

    #[test]
    fn lethal_damage_transitions_to_game_over() {
        let mut app = test_app();
        app.update();

        let player = app
            .world_mut()
            .query_filtered::<Entity, With<Survivor>>()
            .single(app.world())
            .expect("player spawned");

        app.world_mut().write_message(DamageMessage {
            target: player,
            amount: PLAYER_MAX_HP,
            knockback: Vec3::ZERO,
            source: Entity::PLACEHOLDER,
        });

        app.update();
        app.update();

        let state = app.world().resource::<State<GameState>>().get();
        assert_eq!(*state, GameState::GameOver);
    }
}
