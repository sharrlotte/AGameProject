use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use uuid::Uuid;

use crate::config::GameConfig;
use crate::constant::GRID_SIZE;
use crate::content::block::{BlockOwner, BlockRegistry};
use crate::content::lifetime::Lifetime;
use crate::content::team::{GROUP_DEBRIS, GROUP_SHIP, Team};
use crate::resource::SceneState;

use super::Debris;
use crate::content::body::BlockGrid;

pub struct DetachedComponent {
    pub grid: BlockGrid,
    pub blocks: Vec<Entity>,
    pub anchor: IVec2,
}

pub fn detach_component(
    source: &mut BlockGrid,
    ids: &[Uuid],
    registry: &BlockRegistry,
) -> Option<DetachedComponent> {
    let mut anchor = IVec2::splat(i32::MAX);
    let mut placements = Vec::with_capacity(ids.len());

    for id in ids {
        let block = source.block(*id)?;
        let def = registry.get_symbol(&block.placement.block_type)?;
        let footprint = def.shape.get(block.placement.rotation).0;

        for offset in footprint.cells() {
            anchor = anchor.min(block.placement.position + *offset);
        }
        placements.push((block.placement.clone(), block.entity));
    }

    if placements.is_empty() || anchor == IVec2::splat(i32::MAX) {
        return None;
    }

    let mut grid = BlockGrid::new();
    let mut blocks = Vec::with_capacity(placements.len());

    for (placement, entity) in placements {
        let mut shifted = placement;
        shifted.position -= anchor;
        let def = registry.get_symbol(&shifted.block_type)?;
        grid.insert_block(shifted, entity, def);
        blocks.push(entity);
    }

    for id in ids {
        source.remove_block(*id);
    }

    Some(DetachedComponent {
        grid,
        blocks,
        anchor,
    })
}

pub fn component_mass(grid: &BlockGrid, registry: &BlockRegistry) -> f32 {
    grid.blocks()
        .filter_map(|block| registry.get_symbol(&block.placement.block_type))
        .map(|def| def.mass)
        .sum()
}

pub fn component_centroid(source: &BlockGrid, ids: &[Uuid]) -> Vec2 {
    let wanted: std::collections::HashSet<Uuid> = ids.iter().copied().collect();
    let mut sum = Vec2::ZERO;
    let mut count = 0usize;

    for (cell, id) in source.cells() {
        if wanted.contains(id) {
            sum += cell.as_vec2() * GRID_SIZE + Vec2::splat(GRID_SIZE * 0.5);
            count += 1;
        }
    }

    if count == 0 {
        Vec2::ZERO
    } else {
        sum / count as f32
    }
}

pub fn debris_transform(parent: &GlobalTransform, anchor: IVec2) -> Transform {
    let local = Vec3::new(anchor.x as f32 * GRID_SIZE, anchor.y as f32 * GRID_SIZE, 0.0);
    Transform::from_translation(parent.transform_point(local)).with_rotation(parent.rotation())
}

pub fn debris_velocity(
    parent: &Velocity,
    parent_global: &GlobalTransform,
    centroid: Vec2,
) -> Velocity {
    let r = parent_global.transform_point(centroid.extend(0.0)).truncate()
        - parent_global.translation().truncate();

    Velocity {
        linear: parent.linear + parent.angular * Vec2::new(-r.y, r.x),
        angular: parent.angular,
    }
}

pub fn shift_block_transform(transform: &mut Transform, anchor: IVec2) {
    transform.translation.x -= anchor.x as f32 * GRID_SIZE;
    transform.translation.y -= anchor.y as f32 * GRID_SIZE;
}

pub fn next_lifetime(parent: Option<&Lifetime>, config: &GameConfig) -> f32 {
    match parent {
        Some(lifetime) => lifetime.timer().remaining_secs(),
        None => roll_lifetime(config),
    }
}

fn roll_lifetime(config: &GameConfig) -> f32 {
    let min = config.sim.debris_lifetime_min_seconds;
    let max = config.sim.debris_lifetime_max_seconds;
    rand::random::<f32>().mul_add(max - min, min)
}

#[allow(clippy::too_many_arguments)]
pub fn spawn_debris_body(
    commands: &mut Commands,
    grid: BlockGrid,
    transform: Transform,
    velocity: Velocity,
    team: Team,
    lifetime_secs: f32,
    collider: Collider,
    mass: f32,
    config: &GameConfig,
) -> Entity {
    commands
        .spawn((
            Debris,
            grid,
            transform,
            velocity,
            team,
            RigidBody::Dynamic,
            collider,
            ColliderMassProperties::Mass(mass),
            Damping {
                linear_damping: config.sim.linear_damping,
                angular_damping: config.sim.angular_damping,
            },
            CollisionGroups::new(GROUP_DEBRIS, GROUP_SHIP | GROUP_DEBRIS),
            ActiveCollisionTypes::all(),
            Visibility::Visible,
            DespawnOnExit(SceneState::Game),
            Lifetime::seconds(lifetime_secs),
        ))
        .id()
}

pub fn set_block_owner(commands: &mut Commands, block: Entity, body: Entity) {
    commands.entity(block).insert(BlockOwner { body });
}
