use bevy::prelude::*;
use bevy::world_serialization::WorldAssetRoot;
use bevy_rapier3d::prelude::*;
use crate::config::ConfigHotReloadedMessage;
use crate::common::{GROUP_WORLD, Obstacle, GameScene};

use crate::config::{ArenaConfig, ObstacleDef};

use super::map::MapRoot;
use super::state::{ArenaGround, ArenaObstacle, ArenaStaticEntity};

/// Spawns the authored district glTF scene map when entering the arena scene.
pub fn spawn_arena_map(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    existing: Query<(), With<MapRoot>>,
) {
    if !existing.is_empty() {
        return;
    }
    commands.spawn((
        Name::new("District01_Map"),
        MapRoot,
        WorldAssetRoot(asset_server.load("maps/district_01.glb#Scene0")),
        DespawnOnExit(GameScene::Arena),
    ));
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

/// Spawns the flat ground entity scoped to `scene`.
pub fn spawn_ground(commands: &mut Commands, scene: GameScene) {
    commands.spawn((
        Name::new("ArenaGround"),
        ArenaStaticEntity,
        ArenaGround,
        Transform::from_xyz(0.0, -0.5, 0.0),
        DespawnOnExit(scene),
    ));
}

/// Spawns the visual-only ground plane marker and zero-friction static obstacle colliders from `ArenaConfig`.
pub fn spawn_arena_physics(
    mut commands: Commands,
    cfg: Res<ArenaConfig>,
    existing: Query<(), With<ArenaStaticEntity>>,
) {
    if !existing.is_empty() {
        return;
    }
    spawn_arena_entities(&mut commands, &cfg);
}

fn spawn_arena_entities(commands: &mut Commands, cfg: &ArenaConfig) {
    spawn_ground(commands, GameScene::Arena);

    for def in perimeter_walls(cfg)
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
            Obstacle,
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
            DespawnOnExit(GameScene::Arena),
        ));
    }
}

pub fn hot_reload_arena(
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
    spawn_arena_entities(&mut commands, &cfg);
}

