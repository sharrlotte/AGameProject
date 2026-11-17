use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::GameConfigValue;
use crate::{
    constant::GRID_SIZE,
    content::{
        block::{BlockOwner, BlockPlacement, BlockRef, feature::component::Feature},
        ship::{BoundingBox, Ship, ShipBuildResult, ShipFragment, ShipStats, grid::ShipGrid},
        team::Team,
    },
};

#[derive(Clone, PartialEq, Reflect)]
pub struct RecoilComponent {
    pub strength: f32,
}

#[derive(Clone, PartialEq, Reflect)]
pub struct HeatEmitterComponent {
    pub amount: f32,
}

#[derive(Debug, Clone, Reflect, PartialEq, Serialize, Deserialize)]
pub enum TargetingStrategy {
    Closest,
    MostHP,
    EnemyBullet,
}

#[derive(Debug, Clone, Reflect, PartialEq, Serialize, Deserialize)]
pub enum ShootingProjectile {
    Bullet {
        speed: f32,
        damage: f32,
        lifetime: f32,
        piercing: i8,
        width: f32,
        length: f32,
    },
    Laser {
        damage: f32,
        width: f32,
    },
    Missile {
        speed: f32,
        damage: f32,
        lifetime: f32,
        lock_range: f32,
        blast_damage: f32,
        blast_radius: f32,
        width: f32,
        length: f32,
    },
}

#[derive(Component, Debug, Clone, Reflect, PartialEq, Serialize, Deserialize)]
pub struct SingleShootPattern {
    pub fire_rate: f32,
    pub projectile: ShootingProjectile,
}

#[derive(Component, Debug, Clone, Reflect, PartialEq, Serialize, Deserialize)]
pub struct SingleShootPatternState {
    pub last_fired_at: f32,
}

#[derive(Component, Debug, Clone, Reflect, PartialEq)]
pub struct BurstShootPattern {
    pub fire_rate: f32,
    pub delay: f32,
    pub amount: i8,
    pub projectile: ShootingProjectile,
    pub shoot_sound: Option<Handle<AudioSource>>,
}

#[derive(Debug, Clone, Reflect, PartialEq, Serialize, Deserialize)]
pub enum BurstShootPatternShootingState {
    Shooting {
        current_barrel: usize,
        last_barrel_fired_at: f32,
        current_amount: i8,
    },
    Cooldown {
        last_fired_at: f32,
    },
}

#[derive(Component, Debug, Clone, Reflect, PartialEq, Serialize, Deserialize)]
pub struct BurstShootPatternState {
    pub shooting: BurstShootPatternShootingState,
}

#[derive(Component, Debug, Clone, Reflect, PartialEq, Serialize, Deserialize)]
pub struct LaserBeamPattern {
    pub cooldown: f32,
    pub duration: f32,
    pub damage: f32,
    pub width: f32,
}

#[derive(Debug, Clone, Reflect, PartialEq, Serialize, Deserialize)]
pub enum LaserBeamPatternShootingState {
    Shooting { start_time: f32 },
    Cooldown { last_fired_at: f32 },
}

#[derive(Component, Debug, Clone, Reflect, PartialEq, Serialize, Deserialize)]
pub struct PullingBeamPattern {
    pub width: f32,
}

#[derive(Component, Debug, Clone, Reflect, PartialEq, Serialize, Deserialize)]
pub struct PullingBeamPatternState {
    pub beam: Option<Entity>,
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct LaserBeamPatternState {
    pub targets: Vec<Entity>,
    pub beam: Option<Entity>,
    pub shooting: LaserBeamPatternShootingState,
}

#[derive(Debug, Clone, Reflect, PartialEq)]
pub enum ShootPattern {
    Single(SingleShootPattern),
    Burst(BurstShootPattern),
    LaserBeam(LaserBeamPattern),
    PullingBeam(PullingBeamPattern),
}

impl ShootingProjectile {
    pub fn speed(&self) -> f32 {
        match self {
            Self::Bullet { speed, .. } | Self::Missile { speed, .. } => *speed,
            Self::Laser { .. } => f32::INFINITY,
        }
    }
}

impl ShootPattern {
    pub fn projectile_speed(&self) -> f32 {
        match self {
            Self::Single(pattern) => pattern.projectile.speed(),
            Self::Burst(pattern) => pattern.projectile.speed(),
            Self::LaserBeam(_) | Self::PullingBeam(_) => f32::INFINITY,
        }
    }

    pub fn insert(&self, commands: &mut Commands, base: Entity) {
        match self {
            ShootPattern::Single(s) => {
                commands
                    .entity(base)
                    .insert((s.clone(), SingleShootPatternState { last_fired_at: 0.0 }));
            }
            ShootPattern::Burst(s) => {
                commands.entity(base).insert((
                    s.clone(),
                    BurstShootPatternState {
                        shooting: BurstShootPatternShootingState::Cooldown { last_fired_at: 0.0 },
                    },
                ));
            }
            ShootPattern::LaserBeam(s) => {
                commands.entity(base).insert((
                    s.clone(),
                    LaserBeamPatternState {
                        targets: Vec::new(),
                        beam: None,
                        shooting: LaserBeamPatternShootingState::Cooldown { last_fired_at: 0.0 },
                    },
                ));
            }
            ShootPattern::PullingBeam(s) => {
                commands
                    .entity(base)
                    .insert((s.clone(), PullingBeamPatternState { beam: None }));
            }
        }
    }
}

#[derive(Component, Debug, Clone, Default, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Health {
    pub health: f32,
    pub max_health: f32,
}

pub fn spawn_block_entity(
    commands: &mut Commands,
    placement: &BlockPlacement,
    def: &BlockDef,
    ship: Entity,
    team: Team,
    config: &GameConfigValue,
) -> Entity {
    let half_size = def.get_pixel_size() / 2.0;
    let mut transform = def.to_transform(placement.position);

    // Use the first texture's z as the block sprite's render layer so the
    // collapsed sprite keeps its original visual stacking.
    if let Some(z) = def
        .features
        .iter()
        .find_map(|f| f.extra_texture().map(|t| t.z))
    {
        transform.translation.z = z;
    }

    let block = commands
        .spawn((
            transform,
            team,
            Visibility::Visible,
            ActiveEvents::COLLISION_EVENTS,
            Collider::cuboid(half_size.x, half_size.y),
            CollisionGroups::new(team.block_hitbox_group(), team.block_hitbox_filter()),
            ColliderMassProperties::Mass(0.0),
            Sensor,
            BlockRef { id: placement.id },
            BlockOwner { ship },
            Health {
                health: def.health,
                max_health: def.health,
            },
        ))
        .id();

    commands.entity(ship).add_child(block);

    let mut textures = def.features.iter().filter_map(|f| f.extra_texture());
    textures.next();
    for extra in textures {
        let layer = commands
            .spawn((
                Transform::from_translation(extra.offset.extend(extra.z)),
                Sprite {
                    image: extra.image.clone(),
                    custom_size: Some(def.get_pixel_size()),
                    ..default()
                },
            ))
            .id();
        commands.entity(block).add_child(layer);
    }

    for feature in &def.features {
        feature.insert(commands, block, def, &team, config);
    }

    block
}

pub const FACE_RIGHT: u8 = 0b0001;
pub const FACE_LEFT: u8 = 0b0010;
pub const FACE_UP: u8 = 0b0100;
pub const FACE_DOWN: u8 = 0b1000;
pub const FACE_ALL: u8 = 0b1111;

#[derive(Clone, PartialEq, Reflect, Serialize, Deserialize, Debug)]
pub struct BlockVoxel {
    pub position: IVec2,
    pub connectable_faces: u8,
}

impl Default for BlockVoxel {
    fn default() -> Self {
        Self {
            position: IVec2::ZERO,
            connectable_faces: FACE_ALL,
        }
    }
}

#[derive(Clone, PartialEq, Reflect, Serialize, Deserialize, Debug)]
pub struct BlockShape {
    pub size: IVec2,
    pub voxels: Vec<BlockVoxel>,
}

impl Default for BlockShape {
    fn default() -> Self {
        Self {
            size: IVec2::ONE,
            voxels: vec![BlockVoxel::default()],
        }
    }
}

#[derive(Clone, PartialEq, Reflect, Debug, Serialize, Deserialize)]
pub struct ItemDrop {
    pub item_type: String,
    pub amount: u32,
}

#[derive(Component, Clone, PartialEq, Reflect, Serialize, Deserialize, Debug)]
pub enum Loot {
    Item(ItemDrop),
}

#[derive(Message, Clone, PartialEq, Reflect)]
pub struct DropEvent {
    pub drop: Loot,
    pub position: Vec2,
}

#[derive(Clone, PartialEq, Reflect)]
pub struct BlockDef {
    pub name: String,
    pub block_type: String,
    pub block_group: String,
    pub mass: f32,
    pub health: f32,
    pub shape: BlockShape,
    pub cost: u32,
    pub features: Vec<Feature>,
    pub drops: Vec<Loot>,
}

impl BlockDef {
    pub fn get_block_size(&self) -> Vec2 {
        return self.shape.size.as_vec2();
    }

    pub fn get_pixel_size(&self) -> Vec2 {
        return self.get_block_size() * GRID_SIZE;
    }

    pub fn to_transform(&self, position: IVec2) -> Transform {
        let anchor_position = Vec3::new(
            position.x as f32 * GRID_SIZE,
            position.y as f32 * GRID_SIZE,
            0.0,
        );

        let position = anchor_position + (self.get_pixel_size() / 2.0).extend(0.0);

        return Transform::from_translation(position);
    }
}

#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct BlockRegistry {
    pub(crate) map: HashMap<String, BlockDef>,
    pub(crate) by_name: HashMap<String, String>,
    sorted: Vec<String>,
}

impl BlockRegistry {
    pub fn register(&mut self, key: &str, def: BlockDef) {
        if self.map.contains_key(&def.block_type) {
            warn!("Block type {:#?} is already registered.", def.block_type);
            return;
        }

        self.by_name.insert(key.to_string(), def.block_type.clone());
        self.map.insert(def.block_type.clone(), def);

        let mut sorted = self.map.iter().collect::<Vec<_>>();
        sorted.sort_by(|a, b| a.1.block_group.cmp(&b.1.block_group));
        self.sorted = sorted.into_iter().map(|(key, _)| key.clone()).collect();
    }

    pub fn get(&self, key: &str) -> Option<&BlockDef> {
        let sym = self.by_name.get(key)?;
        self.map.get(sym)
    }

    pub fn get_symbol(&self, key: &str) -> Option<&BlockDef> {
        self.map.get(key)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &BlockDef)> {
        self.sorted
            .iter()
            .filter_map(|key| self.map.get_key_value(key))
    }

    pub fn blocks_to_voxel_points(&self, grid: &ShipGrid) -> Vec<Vec2> {
        grid.occupied_cells()
            .map(|(cell, _)| cell.as_vec2() * GRID_SIZE)
            .collect()
    }

    pub fn build_ship_voxel_collider_2d(&self, grid: &ShipGrid) -> Collider {
        let points = self.blocks_to_voxel_points(grid);

        Collider::voxels_from_points(Vec2::new(GRID_SIZE, GRID_SIZE), &points)
    }

    pub fn bounds_from_grid(&self, grid: &ShipGrid) -> Option<BoundingBox> {
        if grid.cells.is_empty() {
            return None;
        }

        let mut min = IVec2::splat(i32::MAX);
        let mut max = IVec2::splat(i32::MIN);

        for cell in grid.cells.keys() {
            min = min.min(*cell);
            max = max.max(*cell + IVec2::ONE);
        }

        let min = min.as_vec2() * GRID_SIZE;
        let max = max.as_vec2() * GRID_SIZE;

        Some(BoundingBox { size: max - min })
    }

    pub fn core_of(&self, grid: &ShipGrid) -> Option<(Uuid, Vec2)> {
        for block in grid.blocks() {
            let def = self.get_symbol(&block.placement.block_type)?;
            if def.block_group == "core" {
                let offset = def.shape.size.as_vec2() * GRID_SIZE * 0.5;
                return Some((
                    block.placement.id,
                    block.placement.position.as_vec2() * GRID_SIZE + offset,
                ));
            }
        }
        None
    }

    pub fn stats_from_grid(&self, grid: &ShipGrid) -> Option<ShipStats> {
        self.core_of(grid)?;

        let mut stats = ShipStats::default();

        for block in grid.blocks() {
            let Some(def) = self.get_symbol(&block.placement.block_type) else {
                warn!(
                    "Block type {:?} not found in registry",
                    block.placement.block_type
                );
                continue;
            };

            stats.mass += def.mass;
            stats.health += def.health;
            stats.max_health += def.health;

            for feature in &def.features {
                match feature {
                    Feature::Turret(turret) => {
                        stats.min_range = if stats.min_range <= 0.0 {
                            turret.range
                        } else {
                            stats.min_range.min(turret.range)
                        };
                        stats.max_range = stats.max_range.max(turret.range);
                    }
                    Feature::Consumer(consumer) => {
                        stats.power_demand += consumer.rate();
                    }
                    Feature::Generator(generator) => {
                        stats.power_production += generator.output;
                    }
                    Feature::Battery(battery) => {
                        stats.power_capacity += battery.capacity;
                    }
                    Feature::Booster(booster) => {
                        stats.power_demand += booster.rate;
                    }
                    Feature::Storage(capacity) => {
                        stats.cargo_capacity += *capacity;
                    }
                    Feature::Thruster(thruster) => {
                        stats.thrust_power += thruster.force();
                        stats.rotate_speed += thruster.torque();
                    }
                    _ => {}
                }
            }
        }

        stats.power_supply = 1.0;
        stats.power_stored = stats.power_capacity;
        Some(stats)
    }

    pub fn build_ship_result(&self, grid: &ShipGrid) -> ShipBuildResult {
        let Some((core_id, core_position)) = self.core_of(grid) else {
            return ShipBuildResult {
                fragments: vec![ShipFragment {
                    placements: grid.blocks().map(|block| block.placement.clone()).collect(),
                }],
                ..default()
            };
        };

        let Some(stats) = self.stats_from_grid(grid) else {
            return default();
        };
        let Some(bounds) = self.bounds_from_grid(grid) else {
            return default();
        };

        ShipBuildResult {
            ship: Some(Ship {
                id: core_id,
                core_position,
            }),
            stats: Some(stats),
            bounds: Some(bounds),
            fragments: Vec::new(),
        }
    }

    pub fn validate_ship(&self, layout: &[BlockPlacement]) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        let mut grid = ShipGrid::new();
        grid.rebuild_from_layout(layout, self);

        let mut core_count = 0;
        for block in grid.blocks() {
            let Some(def) = self.get_symbol(&block.placement.block_type) else {
                errors.push(format!(
                    "unknown block type {:?}",
                    block.placement.block_type
                ));
                continue;
            };
            if def.block_group == "core" {
                core_count += 1;
            }
            for feature in &def.features {
                if let Feature::Turret(turret) = feature
                    && turret.barrels.is_empty()
                {
                    errors.push(format!(
                        "turret {:?} has no barrels",
                        block.placement.block_type
                    ));
                }
            }
        }

        if core_count != 1 {
            errors.push(format!("expected exactly one core, found {core_count}"));
        }

        let mut seen: HashMap<IVec2, Uuid> = HashMap::new();
        for block in grid.blocks() {
            let Some(def) = self.get_symbol(&block.placement.block_type) else {
                continue;
            };
            for y in 0..def.shape.size.y {
                for x in 0..def.shape.size.x {
                    let cell = block.placement.position + IVec2::new(x, y);
                    if let Some(existing) = seen.insert(cell, block.placement.id)
                        && existing != block.placement.id
                    {
                        errors.push(format!(
                            "overlapping blocks at cell {:?} ({existing} and {})",
                            cell, block.placement.id
                        ));
                    }
                }
            }
        }

        if core_count == 1 {
            let core = grid
                .blocks()
                .find(|block| {
                    self.get_symbol(&block.placement.block_type)
                        .map(|def| def.block_group == "core")
                        .unwrap_or(false)
                })
                .map(|block| block.placement.id);

            if let Some(core) = core {
                let mut visited = HashSet::new();
                let mut queue = vec![core];
                while let Some(id) = queue.pop() {
                    if !visited.insert(id) {
                        continue;
                    }
                    let Some(block) = grid.block(id) else {
                        continue;
                    };
                    let Some(def) = self.get_symbol(&block.placement.block_type) else {
                        continue;
                    };
                    for y in 0..def.shape.size.y {
                        for x in 0..def.shape.size.x {
                            let cell = block.placement.position + IVec2::new(x, y);
                            for dir in [IVec2::X, IVec2::NEG_X, IVec2::Y, IVec2::NEG_Y] {
                                if let Some(neighbor) = grid.cells.get(&(cell + dir))
                                    && !visited.contains(neighbor)
                                {
                                    queue.push(*neighbor);
                                }
                            }
                        }
                    }
                }

                if visited.len() != grid.blocks.len() {
                    errors.push(format!(
                        "layout has {} disconnected block(s)",
                        grid.blocks.len() - visited.len()
                    ));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Component, Debug)]
pub struct FlashOverlay {
    pub timer: Timer,
}
