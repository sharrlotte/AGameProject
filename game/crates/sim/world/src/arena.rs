use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use data::{
    AppRonConfigExt, ConfigHotReloadedMessage, GROUP_WORLD, GameState, ObstacleTag, SimUpdateSet,
};

use crate::config::{ArenaConfig, ObstacleDef};

#[derive(Component)]
pub struct ArenaStaticEntity;

#[derive(Component)]
pub struct ArenaGround;

#[derive(Component)]
pub struct ArenaObstacle {
    pub half_extents: Vec3,
    pub is_wall: bool,
}

pub struct ArenaPlugin;

impl Plugin for ArenaPlugin {
    fn build(&self, app: &mut App) {
        app.register_ron_config::<ArenaConfig>()
            .add_systems(OnExit(GameState::Loading), spawn_arena_physics)
            .add_systems(
                Update,
                hot_reload_arena
                    .in_set(SimUpdateSet)
                    .run_if(not(in_state(GameState::Loading))),
            );
    }
}

fn perimeter_walls(cfg: &ArenaConfig) -> [ObstacleDef; 4] {
    let h = cfg.wall_height * 0.5;
    let t = cfg.wall_thickness * 0.5;
    let span = cfg.half_size + t;
    [
        ObstacleDef {
            center: Vec3::new(0.0, h, -span),
            half_extents: Vec3::new(cfg.half_size + cfg.wall_thickness, h, t),
            is_wall: true,
        },
        ObstacleDef {
            center: Vec3::new(0.0, h, span),
            half_extents: Vec3::new(cfg.half_size + cfg.wall_thickness, h, t),
            is_wall: true,
        },
        ObstacleDef {
            center: Vec3::new(-span, h, 0.0),
            half_extents: Vec3::new(t, h, cfg.half_size),
            is_wall: true,
        },
        ObstacleDef {
            center: Vec3::new(span, h, 0.0),
            half_extents: Vec3::new(t, h, cfg.half_size),
            is_wall: true,
        },
    ]
}

/// Spawns the visual-only ground plane marker and zero-friction static obstacle colliders from `ArenaConfig`.
pub fn spawn_arena_physics(mut commands: Commands, cfg: Res<ArenaConfig>) {
    commands.spawn((
        Name::new("ArenaGround"),
        ArenaStaticEntity,
        ArenaGround,
        Transform::from_xyz(0.0, -0.5, 0.0),
    ));

    for def in perimeter_walls(&cfg)
        .into_iter()
        .chain(cfg.obstacles.iter().cloned())
    {
        let label = if def.is_wall {
            "ArenaWall"
        } else {
            "ArenaCover"
        };
        commands.spawn((
            Name::new(label),
            ArenaStaticEntity,
            ObstacleTag,
            ArenaObstacle {
                half_extents: def.half_extents,
                is_wall: def.is_wall,
            },
            RigidBody::Fixed,
            Collider::cuboid(def.half_extents.x, def.half_extents.y, def.half_extents.z),
            Friction {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Min,
            },
            Restitution {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Min,
            },
            CollisionGroups::new(GROUP_WORLD, Group::ALL),
            Transform::from_translation(def.center),
        ));
    }
}

fn hot_reload_arena(
    mut commands: Commands,
    mut reloaded: MessageReader<ConfigHotReloadedMessage>,
    existing: Query<Entity, With<ArenaStaticEntity>>,
    cfg: Res<ArenaConfig>,
) {
    if reloaded.read().next().is_none() || !cfg.is_changed() {
        return;
    }

    for entity in &existing {
        commands.entity(entity).despawn();
    }
    spawn_arena_physics(commands, cfg);
}
