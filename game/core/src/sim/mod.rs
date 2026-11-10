use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::GameState;

pub mod arena;
pub mod combat;
pub mod enemies;
pub mod input_intent;
pub mod player;

pub const GROUP_WORLD: Group = Group::GROUP_1;
pub const GROUP_PLAYER: Group = Group::GROUP_2;
pub const GROUP_ENEMY: Group = Group::GROUP_3;
pub const GROUP_PROJECTILE: Group = Group::GROUP_4;

/// Marks entities spawned for an active run so restarts cleanly despawn them.
#[derive(Component)]
pub struct RunEntity;

#[derive(Resource, Default, Debug)]
pub struct RunStats {
    pub kills: u32,
    pub elapsed_secs: f32,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct DamageMessage {
    pub target: Entity,
    pub amount: f32,
    pub knockback: Vec3,
    #[allow(dead_code)] // Reserved for kill attribution / assists in later stages.
    pub source: Entity,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct EnemyKilledMessage {
    #[allow(dead_code)] // Reserved for loot drops / VFX hooks in Milestone 1.4+.
    pub entity: Entity,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct WeaponFiredMessage {
    #[allow(dead_code)] // Reserved for muzzle flash / audio hooks in Stage 2.
    pub shooter: Entity,
    #[allow(dead_code)]
    pub origin: Vec3,
    #[allow(dead_code)]
    pub direction: Vec3,
}

pub struct SimPlugin;

impl Plugin for SimPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RunStats>()
            .init_resource::<enemies::EnemySpawner>()
            .add_message::<DamageMessage>()
            .add_message::<EnemyKilledMessage>()
            .add_message::<WeaponFiredMessage>()
            .add_systems(Startup, arena::spawn_arena_physics)
            .add_systems(OnExit(GameState::GameOver), cleanup_run_state)
            .add_systems(OnEnter(GameState::Playing), player::spawn_player)
            .add_systems(
                FixedUpdate,
                (
                    tick_run_timer,
                    player::player_movement_and_aim,
                    combat::handle_weapons,
                    enemies::spawn_enemies,
                    enemies::enemy_chase_and_attack,
                )
                    .chain()
                    .before(PhysicsSet::SyncBackend)
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                FixedUpdate,
                (
                    combat::tick_projectiles,
                    combat::handle_projectile_collisions,
                    enemies::apply_enemy_damage,
                    player::apply_player_damage,
                )
                    .chain()
                    .after(PhysicsSet::Writeback)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn tick_run_timer(time: Res<Time<Fixed>>, mut stats: ResMut<RunStats>) {
    stats.elapsed_secs += time.delta_secs();
}

fn cleanup_run_state(
    mut commands: Commands,
    run_entities: Query<Entity, With<RunEntity>>,
    mut stats: ResMut<RunStats>,
    mut spawner: ResMut<enemies::EnemySpawner>,
) {
    for entity in &run_entities {
        commands.entity(entity).despawn();
    }
    *stats = RunStats::default();
    *spawner = enemies::EnemySpawner::default();
}
