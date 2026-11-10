use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use super::GROUP_WORLD;

pub const ARENA_HALF_SIZE: f32 = 25.0;
pub const WALL_HEIGHT: f32 = 2.6;
pub const WALL_THICKNESS: f32 = 1.0;

/// Marker for the static arena floor collider so presentation can attach its 3D mesh.
#[derive(Component)]
pub struct ArenaGround;

/// Marker and dimensions for static arena walls and cover blocks.
#[derive(Component)]
pub struct ArenaObstacle {
    pub half_extents: Vec3,
    pub is_wall: bool,
}

#[derive(Clone, Copy)]
struct ObstacleDef {
    center: Vec3,
    half_extents: Vec3,
    is_wall: bool,
}

fn arena_obstacle_defs() -> Vec<ObstacleDef> {
    let h = WALL_HEIGHT * 0.5;
    let t = WALL_THICKNESS * 0.5;
    let span = ARENA_HALF_SIZE + t;

    vec![
        // 4 perimeter walls
        ObstacleDef {
            center: Vec3::new(0.0, h, -span),
            half_extents: Vec3::new(ARENA_HALF_SIZE + WALL_THICKNESS, h, t),
            is_wall: true,
        },
        ObstacleDef {
            center: Vec3::new(0.0, h, span),
            half_extents: Vec3::new(ARENA_HALF_SIZE + WALL_THICKNESS, h, t),
            is_wall: true,
        },
        ObstacleDef {
            center: Vec3::new(-span, h, 0.0),
            half_extents: Vec3::new(t, h, ARENA_HALF_SIZE),
            is_wall: true,
        },
        ObstacleDef {
            center: Vec3::new(span, h, 0.0),
            half_extents: Vec3::new(t, h, ARENA_HALF_SIZE),
            is_wall: true,
        },
        // 4 inner cover walls around the center spawn
        ObstacleDef {
            center: Vec3::new(-7.5, 0.9, -6.5),
            half_extents: Vec3::new(2.2, 0.9, 0.65),
            is_wall: false,
        },
        ObstacleDef {
            center: Vec3::new(7.5, 0.9, -6.5),
            half_extents: Vec3::new(2.2, 0.9, 0.65),
            is_wall: false,
        },
        ObstacleDef {
            center: Vec3::new(-7.5, 0.9, 6.5),
            half_extents: Vec3::new(2.2, 0.9, 0.65),
            is_wall: false,
        },
        ObstacleDef {
            center: Vec3::new(7.5, 0.9, 6.5),
            half_extents: Vec3::new(2.2, 0.9, 0.65),
            is_wall: false,
        },
        // 4 mid-lane cover blocks
        ObstacleDef {
            center: Vec3::new(-14.5, 0.85, 0.0),
            half_extents: Vec3::new(1.1, 0.85, 1.8),
            is_wall: false,
        },
        ObstacleDef {
            center: Vec3::new(14.5, 0.85, 0.0),
            half_extents: Vec3::new(1.1, 0.85, 1.8),
            is_wall: false,
        },
        ObstacleDef {
            center: Vec3::new(0.0, 0.85, -14.5),
            half_extents: Vec3::new(1.8, 0.85, 1.1),
            is_wall: false,
        },
        ObstacleDef {
            center: Vec3::new(0.0, 0.85, 14.5),
            half_extents: Vec3::new(1.8, 0.85, 1.1),
            is_wall: false,
        },
    ]
}

/// Spawns the visual-only ground plane marker and static 3D obstacle colliders.
pub fn spawn_arena_physics(mut commands: Commands) {
    commands.spawn((
        Name::new("ArenaGround"),
        ArenaGround,
        Transform::from_xyz(0.0, -0.5, 0.0),
    ));

    for def in arena_obstacle_defs() {
        let label = if def.is_wall { "ArenaWall" } else { "ArenaCover" };
        commands.spawn((
            Name::new(label),
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
