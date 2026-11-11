use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use data::{
    AppRonConfigExt, ConfigHotReloadedMessage, DamageMessage, GROUP_ENEMY, GROUP_PICKUP,
    GROUP_PLAYER, GROUP_WORLD, GameState, PlayerDiedMessage, RunEntity, SimPostPhysicsSet,
    SimUpdateSet, SurvivorTag, UpgradeModifiers,
};

use crate::config::PlayerConfig;
use crate::input_intent::ActionIntent;

#[derive(Component)]
pub struct Survivor;

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

impl Stamina {
    pub fn new(max: f32) -> Self {
        Self {
            current: max,
            max,
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

#[derive(Component, Debug, Clone, Copy)]
pub struct AimDirection(pub Vec3);

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_ron_config::<PlayerConfig>()
            .add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(OnEnter(GameState::GameOver), freeze_survivor_on_game_over)
            .add_systems(
                Update,
                (sync_player_hot_reload, player_movement_and_aim)
                    .chain()
                    .in_set(SimUpdateSet)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                PostUpdate,
                apply_player_damage
                    .in_set(SimPostPhysicsSet)
                    .after(PhysicsSet::Writeback)
                    .before(TransformSystems::Propagate)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

pub fn spawn_player(
    mut commands: Commands,
    cfg: Res<PlayerConfig>,
    upgrades: Res<UpgradeModifiers>,
) {
    let max_hp = cfg.max_hp + upgrades.bonus_max_hp;
    commands.spawn((
        (
            Name::new("Survivor"),
            Survivor,
            SurvivorTag,
            LocalPlayer,
            RunEntity,
            ActionIntent::default(),
            Health::new(max_hp),
            Stamina::new(cfg.max_stamina),
            MovementStats {
                walk_speed: cfg.walk_speed * upgrades.move_speed_multiplier,
                sprint_speed: cfg.sprint_speed * upgrades.move_speed_multiplier,
            },
            AimDirection(Vec3::NEG_Z),
        ),
        (
            RigidBody::Dynamic,
            Collider::capsule_y(cfg.half_height, cfg.radius),
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
            CollisionGroups::new(GROUP_PLAYER, GROUP_WORLD | GROUP_ENEMY | GROUP_PICKUP),
            Transform::from_xyz(0.0, cfg.aim_height, 0.0),
        ),
    ));
}

fn sync_player_hot_reload(
    mut reloaded: MessageReader<ConfigHotReloadedMessage>,
    cfg: Res<PlayerConfig>,
    upgrades: Res<UpgradeModifiers>,
    mut players: Query<(&mut Health, &mut Stamina, &mut MovementStats), With<Survivor>>,
) {
    if reloaded.read().next().is_none() || !cfg.is_changed() {
        return;
    }

    let target_max_hp = cfg.max_hp + upgrades.bonus_max_hp;
    for (mut health, mut stamina, mut stats) in &mut players {
        let hp_delta = target_max_hp - health.max;
        health.max = target_max_hp;
        health.current = (health.current + hp_delta.max(0.0)).clamp(0.0, target_max_hp);
        stamina.max = cfg.max_stamina;
        stamina.current = stamina.current.min(cfg.max_stamina);
        stats.walk_speed = cfg.walk_speed * upgrades.move_speed_multiplier;
        stats.sprint_speed = cfg.sprint_speed * upgrades.move_speed_multiplier;
    }
}

#[allow(clippy::type_complexity)]
pub fn player_movement_and_aim(
    time: Res<Time>,
    cfg: Res<PlayerConfig>,
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
            stamina.current = (stamina.current - cfg.stamina_drain_per_sec * dt).max(0.0);
            if stamina.current <= 0.0 {
                stamina.exhausted_timer = cfg.stamina_exhaustion_delay;
            }
        } else {
            if stamina.exhausted_timer > 0.0 {
                stamina.exhausted_timer = (stamina.exhausted_timer - dt).max(0.0);
            }
            if stamina.exhausted_timer <= 0.0 {
                stamina.current =
                    (stamina.current + cfg.stamina_regen_per_sec * dt).min(stamina.max);
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

pub fn apply_player_damage(
    mut damage_reader: MessageReader<DamageMessage>,
    mut died_writer: MessageWriter<PlayerDiedMessage>,
    mut survivors: Query<(Entity, &mut Health), With<Survivor>>,
) {
    for msg in damage_reader.read() {
        let Ok((entity, mut health)) = survivors.get_mut(msg.target) else {
            continue;
        };
        if health.current <= 0.0 {
            continue;
        }
        health.current = (health.current - msg.amount).max(0.0);
        if health.current <= 0.0 {
            died_writer.write(PlayerDiedMessage { survivor: entity });
        }
    }
}

fn freeze_survivor_on_game_over(
    mut survivors: Query<(&mut Velocity, &mut Stamina), With<Survivor>>,
) {
    for (mut vel, mut stamina) in &mut survivors {
        vel.linear = Vec3::ZERO;
        vel.angular = Vec3::ZERO;
        stamina.is_sprinting = false;
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<PlayerConfig>()
            .init_resource::<UpgradeModifiers>()
            .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
                Duration::from_secs_f32(1.0 / 60.0),
            ))
            .add_message::<DamageMessage>()
            .add_message::<PlayerDiedMessage>()
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
            intent.move_dir = Vec2::new(1.0, 1.0);
            intent.sprint = true;
            intent.aim_world = Some(Vec3::new(10.0, 0.9, 0.0));
        }

        for _ in 0..30 {
            app.update();
        }

        let cfg = app.world().resource::<PlayerConfig>();
        let expected_sprint = cfg.sprint_speed;
        let expected_stamina = cfg.max_stamina;

        let vel = app.world().get::<Velocity>(player).unwrap();
        let speed = Vec2::new(vel.linear.x, vel.linear.z).length();
        assert!((speed - expected_sprint).abs() < 0.01);

        let stamina = app.world().get::<Stamina>(player).unwrap();
        assert!(stamina.current < expected_stamina);
        assert!(stamina.is_sprinting);

        let aim = app.world().get::<AimDirection>(player).unwrap();
        assert!((aim.0 - Vec3::X).length() < 0.01);
    }

    #[test]
    fn lethal_damage_emits_player_died_message() {
        let mut app = test_app();
        app.update();

        let player = app
            .world_mut()
            .query_filtered::<Entity, With<Survivor>>()
            .single(app.world())
            .expect("player spawned");

        let max_hp = app.world().resource::<PlayerConfig>().max_hp;
        app.world_mut().write_message(DamageMessage {
            target: player,
            amount: max_hp,
            knockback: Vec3::ZERO,
            source: Entity::PLACEHOLDER,
        });

        app.update();
        app.update();

        let msgs = app.world().resource::<Messages<PlayerDiedMessage>>();
        let mut cursor = msgs.get_cursor();
        let died: Vec<_> = cursor.read(msgs).copied().collect();
        assert_eq!(died.len(), 1);
        assert_eq!(died[0].survivor, player);
    }
}
