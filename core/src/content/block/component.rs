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
        body::BlockGrid,
        ship::{BoundingBox, Ship, GridBuildResult, ShipStats},
        team::Team,
    },
};


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize, Default)]
pub enum Rot90 {
    #[default]
    R0,
    R90,
    R180,
    R270,
}


impl Rot90 {
    pub const ALL: [Self; 4] = [Self::R0, Self::R90, Self::R180, Self::R270];

    pub fn turns(self) -> usize {
        match self {
            Self::R0 => 0,
            Self::R90 => 1,
            Self::R180 => 2,
            Self::R270 => 3,
        }
    }

    pub fn ccw(self) -> Self {
        match self {
            Self::R0 => Self::R90,
            Self::R90 => Self::R180,
            Self::R180 => Self::R270,
            Self::R270 => Self::R0,
        }
    }

    pub fn cw(self) -> Self {
        match self {
            Self::R0 => Self::R270,
            Self::R90 => Self::R0,
            Self::R180 => Self::R90,
            Self::R270 => Self::R180,
        }
    }

    fn rotate_cell(self, cell: IVec2) -> IVec2 {
        match self {
            Self::R0 => cell,
            Self::R90 => IVec2::new(-cell.y, cell.x),
            Self::R180 => -cell,
            Self::R270 => IVec2::new(cell.y, -cell.x),
        }
    }

    fn rotate_vec(self, v: Vec2) -> Vec2 {
        match self {
            Self::R0 => v,
            Self::R90 => Vec2::new(-v.y, v.x),
            Self::R180 => -v,
            Self::R270 => Vec2::new(v.y, -v.x),
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Serialize, Deserialize)]
#[serde(default)]
pub struct Xform {
    pub rotate: Rot90,
    pub flip_x: bool,
    pub flip_y: bool,
}


impl Default for Xform {
    fn default() -> Self {
        Self {
            rotate: Rot90::R0,
            flip_x: false,
            flip_y: false,
        }
    }
}


impl Xform {
    fn apply_cell(self, cell: IVec2) -> IVec2 {
        let cell = self.rotate.rotate_cell(cell);
        let cell = if self.flip_x {
            IVec2::new(-cell.x, cell.y)
        } else {
            cell
        };
        if self.flip_y {
            IVec2::new(cell.x, -cell.y)
        } else {
            cell
        }
    }

    fn apply_vec(self, v: Vec2) -> Vec2 {
        let v = self.rotate.rotate_vec(v);
        let v = if self.flip_x { Vec2::new(-v.x, v.y) } else { v };
        if self.flip_y { Vec2::new(v.x, -v.y) } else { v }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Serialize, Deserialize, Default)]
pub enum TriangleBase {
    #[default]
    Right,
    Isosceles,
}


#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub enum ShapeSpec {
    Square(u32),
    Rect(UVec2),
    Triangle {
        base: TriangleBase,
        size: UVec2,
        #[serde(default)]
        xform: Xform,
    },
    Rows {
        art: Vec<String>,
        #[serde(default)]
        xform: Xform,
    },
}


impl ShapeSpec {
    pub fn xform(&self) -> Xform {
        match self {
            Self::Triangle { xform, .. } | Self::Rows { xform, .. } => *xform,
            Self::Square(_) | Self::Rect(_) => Xform::default(),
        }
    }
}


#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct Footprint {
    cells: Vec<IVec2>,
    size: IVec2,
}


impl Footprint {
    pub fn cells(&self) -> &[IVec2] {
        &self.cells
    }

    pub fn size(&self) -> IVec2 {
        self.size
    }
}


#[derive(Debug, Clone, PartialEq, Reflect)]
pub enum Hitbox {
    Cuboid(Vec2),
    Polygon(Vec<Vec2>),
    Cells(Vec<IVec2>),
}


#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct BakedShape {
    rotations: [(Footprint, Hitbox); 4],
    base: Footprint,
    xform: Xform,
}


impl BakedShape {
    pub fn get(&self, rot: Rot90) -> (&Footprint, &Hitbox) {
        let (footprint, hitbox) = &self.rotations[rot.turns()];
        (footprint, hitbox)
    }

    pub fn base(&self) -> &Footprint {
        &self.base
    }

    pub fn xform(&self) -> Xform {
        self.xform
    }
}


impl TryFrom<&ShapeSpec> for BakedShape {
    type Error = ShapeError;

    fn try_from(spec: &ShapeSpec) -> Result<Self, Self::Error> {
        let base = make_footprint(base_cells(spec)?)?;
        let xform = spec.xform();
        let vertices = match spec {
            ShapeSpec::Triangle { base, size, .. } => Some(triangle_vertices(*base, *size)),
            ShapeSpec::Square(_) | ShapeSpec::Rect(_) | ShapeSpec::Rows { .. } => None,
        };

        let mut baked = Vec::with_capacity(Rot90::ALL.len());
        for rot in Rot90::ALL {
            let cells: Vec<IVec2> = base
                .cells()
                .iter()
                .map(|cell| rot.rotate_cell(xform.apply_cell(*cell)))
                .collect();
            let footprint = make_footprint(cells)?;

            let hitbox = match spec {
                ShapeSpec::Square(_) | ShapeSpec::Rect(_) => {
                    Hitbox::Cuboid(footprint.size().as_vec2() * GRID_SIZE * 0.5)
                }
                ShapeSpec::Rows { .. } => Hitbox::Cells(footprint.cells().to_vec()),
                ShapeSpec::Triangle { .. } => {
                    let Some(source) = vertices else {
                        return Err(ShapeError::new("triangle has no vertices"));
                    };
                    let points: Vec<Vec2> = source
                        .iter()
                        .map(|v| rot.rotate_vec(xform.apply_vec(*v)) * GRID_SIZE)
                        .collect();
                    if Collider::convex_hull(&points).is_none() {
                        return Err(ShapeError::new("triangle hitbox has no convex hull"));
                    }
                    Hitbox::Polygon(points)
                }
            };

            baked.push((footprint, hitbox));
        }

        let rotations: [(Footprint, Hitbox); 4] = baked
            .try_into()
            .map_err(|_| ShapeError::new("failed to bake shape rotations"))?;

        Ok(Self {
            rotations,
            base,
            xform,
        })
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShapeError {
    message: String,
}


impl ShapeError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}


impl std::fmt::Display for ShapeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}


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


#[derive(Component, Debug, Clone, Default, Reflect, PartialEq, Serialize, Deserialize)]
pub struct SingleShootPatternState {
    pub cooldown_remaining: f32,
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
        barrel_delay_remaining: f32,
        current_amount: i8,
    },
    Cooldown {
        cooldown_remaining: f32,
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
    Shooting { elapsed: f32 },
    Cooldown { cooldown_remaining: f32 },
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
                commands.entity(base).insert((
                    s.clone(),
                    SingleShootPatternState {
                        cooldown_remaining: 0.0,
                    },
                ));
            }
            ShootPattern::Burst(s) => {
                commands.entity(base).insert((
                    s.clone(),
                    BurstShootPatternState {
                        shooting: BurstShootPatternShootingState::Cooldown {
                            cooldown_remaining: 0.0,
                        },
                    },
                ));
            }
            ShootPattern::LaserBeam(s) => {
                commands.entity(base).insert((
                    s.clone(),
                    LaserBeamPatternState {
                        beam: None,
                        shooting: LaserBeamPatternShootingState::Cooldown {
                            cooldown_remaining: 0.0,
                        },
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
    pub shape: BakedShape,
    pub cost: u32,
    pub features: Vec<Feature>,
    pub drops: Vec<Loot>,
}


impl BlockDef {
    pub fn get_block_size(&self, rot: Rot90) -> Vec2 {
        self.shape.get(rot).0.size().as_vec2()
    }

    pub fn get_pixel_size(&self, rot: Rot90) -> Vec2 {
        self.get_block_size(rot) * GRID_SIZE
    }

    pub fn get_base_pixel_size(&self) -> Vec2 {
        self.shape.base().size().as_vec2() * GRID_SIZE
    }

    pub fn to_transform(&self, position: IVec2, rot: Rot90) -> Transform {
        let anchor_position = Vec3::new(
            position.x as f32 * GRID_SIZE,
            position.y as f32 * GRID_SIZE,
            0.0,
        );

        let position = anchor_position + (self.get_pixel_size(rot) / 2.0).extend(0.0);

        Transform::from_translation(position).with_rotation(Quat::from_rotation_z(
            rot.turns() as f32 * std::f32::consts::FRAC_PI_2,
        ))
    }

    pub fn solid_edge(&self, rot: Rot90, offset: IVec2, dir: IVec2) -> bool {
        let (footprint, hitbox) = self.shape.get(rot);
        let Hitbox::Polygon(points) = hitbox else {
            return true;
        };

        let center = footprint.size().as_vec2() * (GRID_SIZE * 0.5);
        let local = |x: f32, y: f32| Vec2::new(x - center.x, y - center.y);
        let left = offset.x as f32 * GRID_SIZE;
        let right = left + GRID_SIZE;
        let bottom = offset.y as f32 * GRID_SIZE;
        let top = bottom + GRID_SIZE;

        let (start, end) = match dir {
            IVec2::X => (Vec2::new(right, bottom), Vec2::new(right, top)),
            IVec2::NEG_X => (Vec2::new(left, bottom), Vec2::new(left, top)),
            IVec2::Y => (Vec2::new(left, top), Vec2::new(right, top)),
            IVec2::NEG_Y => (Vec2::new(left, bottom), Vec2::new(right, bottom)),
            _ => return false,
        };

        point_in_convex(local(start.x, start.y), points)
            && point_in_convex(local(end.x, end.y), points)
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

    pub fn build_voxel_collider_2d(&self, grid: &BlockGrid) -> Collider {
        let mut parts: Vec<(Vec2, f32, Collider)> = Vec::new();

        for block in grid.blocks() {
            let Some(def) = self.get_symbol(&block.placement.block_type) else {
                continue;
            };
            let (footprint, hitbox) = def.shape.get(block.placement.rotation);
            let center = block.placement.position.as_vec2() * GRID_SIZE
                + footprint.size().as_vec2() * GRID_SIZE * 0.5;
            parts.push((center, 0.0, collider_from_hitbox(hitbox)));
        }

        Collider::compound(parts)
    }

    pub fn bounds_from_grid(&self, grid: &BlockGrid) -> Option<BoundingBox> {
        if grid.cells().is_empty() {
            return None;
        }

        let mut min = IVec2::splat(i32::MAX);
        let mut max = IVec2::splat(i32::MIN);

        for cell in grid.cells().keys() {
            min = min.min(*cell);
            max = max.max(*cell + IVec2::ONE);
        }

        let min = min.as_vec2() * GRID_SIZE;
        let max = max.as_vec2() * GRID_SIZE;

        Some(BoundingBox::from_min_max(min, max))
    }

    pub fn core_of(&self, grid: &BlockGrid) -> Option<(Uuid, Vec2)> {
        for block in grid.blocks() {
            let def = self.get_symbol(&block.placement.block_type)?;
            if def.block_group == "core" {
                let offset = def.get_pixel_size(block.placement.rotation) * 0.5;
                return Some((
                    block.placement.id,
                    block.placement.position.as_vec2() * GRID_SIZE + offset,
                ));
            }
        }
        None
    }

    pub fn stats_from_grid(&self, grid: &BlockGrid) -> Option<ShipStats> {
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
                    Feature::Shield(shield) => {
                        if let Some(demand) = shield.demand {
                            stats.power_demand += demand;
                        }
                    }
                    _ => {}
                }
            }
        }

        stats.power_supply = 1.0;
        stats.power_stored = stats.power_capacity;
        Some(stats)
    }

    pub fn build_grid_result(&self, grid: &BlockGrid) -> Option<GridBuildResult> {
        let (core_id, core_position) = self.core_of(grid)?;
        let stats = self.stats_from_grid(grid)?;
        let bounds = self.bounds_from_grid(grid)?;

        Some(GridBuildResult {
            ship: Ship {
                id: Uuid::now_v7(),
                core_id,
                core_position,
            },
            stats,
            bounds,
        })
    }

    pub fn validate_ship(&self, layout: &[BlockPlacement]) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        let mut grid = BlockGrid::new();
        
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
            let footprint = def.shape.get(block.placement.rotation).0;
            for offset in footprint.cells() {
                let cell = block.placement.position + *offset;
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

        for (cell, source_id) in grid.cells() {
            for dir in [IVec2::X, IVec2::NEG_X, IVec2::Y, IVec2::NEG_Y] {
                let Some(neighbor_id) = grid.cells().get(&(*cell + dir)) else {
                    continue;
                };
                if source_id == neighbor_id {
                    continue;
                }
                if grid.solid_neighbor(self, *cell, dir).is_none() {
                    errors.push(format!(
                        "block at cell {:?} touches a neighbor on an empty edge",
                        cell
                    ));
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
                    let footprint = def.shape.get(block.placement.rotation).0;
                    for offset in footprint.cells() {
                        let cell = block.placement.position + *offset;
                        for dir in [IVec2::X, IVec2::NEG_X, IVec2::Y, IVec2::NEG_Y] {
                            if let Some(neighbor) = grid.solid_neighbor(self, cell, dir)
                                && !visited.contains(&neighbor)
                            {
                                queue.push(neighbor);
                            }
                        }
                    }
                }

                if visited.len() != grid.block_count() {
                    errors.push(format!(
                        "layout has {} disconnected block(s)",
                        grid.block_count() - visited.len()
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


#[derive(Component, Debug, Clone, Copy)]
pub struct Visual;


#[derive(Component, Debug, Clone, Copy)]
pub struct TurretMirror(pub bool);


fn base_cells(spec: &ShapeSpec) -> Result<Vec<IVec2>, ShapeError> {
    match spec {
        ShapeSpec::Square(n) => {
            if *n == 0 {
                return Err(ShapeError::new("Square size must be positive"));
            }
            Ok(grid_cells(*n, *n))
        }
        ShapeSpec::Rect(size) => {
            if size.x == 0 || size.y == 0 {
                return Err(ShapeError::new("Rect dimensions must be positive"));
            }
            Ok(grid_cells(size.x, size.y))
        }
        ShapeSpec::Triangle { base, size, .. } => triangle_cells(*base, *size),
        ShapeSpec::Rows { art, .. } => rows_cells(art),
    }
}


fn grid_cells(width: u32, height: u32) -> Vec<IVec2> {
    let mut cells = Vec::with_capacity((width * height) as usize);
    for y in 0..height as i32 {
        for x in 0..width as i32 {
            cells.push(IVec2::new(x, y));
        }
    }
    cells
}


fn triangle_cells(base: TriangleBase, size: UVec2) -> Result<Vec<IVec2>, ShapeError> {
    if size.x == 0 || size.y == 0 {
        return Err(ShapeError::new("Triangle dimensions must be positive"));
    }

    let w = size.x as i64;
    let h = size.y as i64;
    let mut cells = Vec::new();

    for y in 0..h {
        for x in 0..w {
            let included = match base {
                TriangleBase::Right => (2 * x + 1) * h + (2 * y + 1) * w <= 2 * w * h,
                TriangleBase::Isosceles => {
                    let yy = 2 * y + 1;
                    2 * h * (2 * x + 1) >= w * yy && 2 * h * (2 * w - 2 * x - 1) >= w * yy
                }
            };
            if included {
                cells.push(IVec2::new(x as i32, y as i32));
            }
        }
    }

    let size = footprint_size(&cells);
    if size != IVec2::new(w as i32, h as i32) {
        return Err(ShapeError::new(
            "triangle raster does not span its declared size",
        ));
    }

    Ok(cells)
}


fn rows_cells(art: &[String]) -> Result<Vec<IVec2>, ShapeError> {
    let Some(first) = art.first() else {
        return Err(ShapeError::new("Rows has no rows"));
    };
    let width = first.len();
    if width == 0 {
        return Err(ShapeError::new("Rows has no columns"));
    }
    if art.iter().any(|row| row.len() != width) {
        return Err(ShapeError::new("Rows are ragged"));
    }

    let mut cells = Vec::new();
    for (y, row) in art.iter().enumerate() {
        for (x, character) in row.chars().enumerate() {
            match character {
                '#' => cells.push(IVec2::new(x as i32, y as i32)),
                '.' => {}
                _ => return Err(ShapeError::new("Rows contains an invalid character")),
            }
        }
    }

    if cells.is_empty() {
        return Err(ShapeError::new("Rows has no filled cells"));
    }

    let last_row = art.len() - 1;
    let last_column = width - 1;
    let empty_border = !cells.iter().any(|c| c.y == 0)
        || !cells.iter().any(|c| c.y == last_row as i32)
        || !cells.iter().any(|c| c.x == 0)
        || !cells.iter().any(|c| c.x == last_column as i32);
    if empty_border {
        return Err(ShapeError::new("Rows has an empty border"));
    }

    Ok(cells)
}


fn triangle_vertices(base: TriangleBase, size: UVec2) -> [Vec2; 3] {
    let width = size.x as f32;
    let height = size.y as f32;
    let center = Vec2::new(width * 0.5, height * 0.5);
    let origin = Vec2::new(0.0, 0.0) - center;
    let bottom_right = Vec2::new(width, 0.0) - center;
    let apex = match base {
        TriangleBase::Right => Vec2::new(0.0, height) - center,
        TriangleBase::Isosceles => Vec2::new(width * 0.5, height) - center,
    };
    [origin, bottom_right, apex]
}


fn footprint_size(cells: &[IVec2]) -> IVec2 {
    let min = cells
        .iter()
        .fold(IVec2::MAX, |accumulator, cell| accumulator.min(*cell));
    let max = cells
        .iter()
        .fold(IVec2::MIN, |accumulator, cell| accumulator.max(*cell));
    max - min + IVec2::ONE
}


fn make_footprint(mut cells: Vec<IVec2>) -> Result<Footprint, ShapeError> {
    if cells.is_empty() {
        return Err(ShapeError::new("shape has no cells"));
    }
    if !connected(&cells) {
        return Err(ShapeError::new("shape cells are disconnected"));
    }

    let min = cells
        .iter()
        .fold(IVec2::MAX, |accumulator, cell| accumulator.min(*cell));
    for cell in &mut cells {
        *cell -= min;
    }

    Ok(Footprint {
        size: footprint_size(&cells),
        cells,
    })
}


fn point_in_convex(point: Vec2, polygon: &[Vec2]) -> bool {
    let mut winding = 0.0;
    for (index, start) in polygon.iter().enumerate() {
        let end = polygon[(index + 1) % polygon.len()];
        let cross = (end.x - start.x) * (point.y - start.y)
            - (end.y - start.y) * (point.x - start.x);
        if cross.abs() < 1.0e-3 {
            continue;
        }
        let sign = cross.signum();
        if winding == 0.0 {
            winding = sign;
        } else if winding != sign {
            return false;
        }
    }
    true
}


fn connected(cells: &[IVec2]) -> bool {
    let Some(&start) = cells.first() else {
        return false;
    };
    let mut visited = HashSet::new();
    visited.insert(start);
    let mut queue = vec![start];

    while let Some(cell) = queue.pop() {
        for direction in [IVec2::X, IVec2::NEG_X, IVec2::Y, IVec2::NEG_Y] {
            let neighbor = cell + direction;
            if cells.contains(&neighbor) && visited.insert(neighbor) {
                queue.push(neighbor);
            }
        }
    }

    visited.len() == cells.len()
}


fn collider_from_hitbox(hitbox: &Hitbox) -> Collider {
    match hitbox {
        Hitbox::Cuboid(half) => Collider::cuboid(half.x, half.y),
        Hitbox::Polygon(points) => Collider::convex_hull(points).unwrap_or_else(|| {
            error!("failed to build a convex hull for a baked triangle hitbox");
            Collider::cuboid(0.0, 0.0)
        }),
        Hitbox::Cells(cells) => {
            let max = cells
                .iter()
                .fold(IVec2::MIN, |accumulator, cell| accumulator.max(*cell));
            let size = (max + IVec2::ONE).as_vec2();
            let parts: Vec<(Vec2, f32, Collider)> = cells
                .iter()
                .map(|cell| {
                    let center = (cell.as_vec2() + Vec2::splat(0.5) - size * 0.5) * GRID_SIZE;
                    (
                        center,
                        0.0,
                        Collider::cuboid(GRID_SIZE * 0.5, GRID_SIZE * 0.5),
                    )
                })
                .collect();
            Collider::compound(parts)
        }
    }
}


pub fn visual_transform(xform: Xform) -> Transform {
    let swap = matches!(xform.rotate, Rot90::R90 | Rot90::R270);
    let (flip_x, flip_y) = if swap {
        (xform.flip_y, xform.flip_x)
    } else {
        (xform.flip_x, xform.flip_y)
    };
    Transform {
        rotation: Quat::from_rotation_z(
            xform.rotate.turns() as f32 * std::f32::consts::FRAC_PI_2,
        ),
        scale: Vec3::new(
            if flip_x { -1.0 } else { 1.0 },
            if flip_y { -1.0 } else { 1.0 },
            1.0,
        ),
        ..default()
    }
}


pub fn spawn_block_entity(
    commands: &mut Commands,
    placement: &BlockPlacement,
    def: &BlockDef,
    body: Entity,
    team: Team,
    config: &GameConfigValue,
) -> Entity {
    let base_pixel_size = def.get_base_pixel_size();
    let (_, hitbox) = def.shape.get(placement.rotation);
    let xform = def.shape.xform();
    let mirror = xform.flip_x ^ xform.flip_y;
    let mut transform = def.to_transform(placement.position, placement.rotation);

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
            Visibility::Inherited,
            ActiveEvents::COLLISION_EVENTS,
            collider_from_hitbox(hitbox),
            CollisionGroups::new(team.block_hitbox_group(), team.block_hitbox_filter()),
            ColliderMassProperties::Mass(0.0),
            Sensor,
            BlockRef { id: placement.id },
            BlockOwner { body },
            TurretMirror(mirror),
            Health {
                health: def.health,
                max_health: def.health,
            },
        ))
        .id();

    commands.entity(body).add_child(block);

    let visual = commands
        .spawn((visual_transform(xform), Visual, Visibility::Inherited))
        .id();
    #[cfg(feature = "dev")]
    commands.entity(visual).insert(Pickable::default());
    commands.entity(block).add_child(visual);

    let mut textures = def.features.iter().filter_map(|f| f.extra_texture());
    textures.next();
    for extra in textures {
        let layer = commands
            .spawn((
                Transform::from_translation(extra.offset.extend(extra.z)),
                Sprite {
                    image: extra.image.clone(),
                    custom_size: Some(base_pixel_size),
                    ..default()
                },
            ))
            .id();
        commands.entity(visual).add_child(layer);
    }

    for feature in &def.features {
        feature.insert(commands, block, visual, base_pixel_size, &team, config);
    }

    block
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bake(spec: &ShapeSpec) -> BakedShape {
        BakedShape::try_from(spec).expect("shape bakes")
    }

    fn rows(art: &[&str]) -> ShapeSpec {
        ShapeSpec::Rows {
            art: art.iter().map(|row| row.to_string()).collect(),
            xform: Xform::default(),
        }
    }

    #[test]
    fn square_and_rect_cells() {
        let square = bake(&ShapeSpec::Square(2));
        assert_eq!(square.get(Rot90::R0).0.size(), IVec2::new(2, 2));
        assert_eq!(square.get(Rot90::R0).0.cells().len(), 4);

        let rect = bake(&ShapeSpec::Rect(UVec2::new(3, 5)));
        assert_eq!(rect.get(Rot90::R0).0.size(), IVec2::new(3, 5));
        assert_eq!(rect.get(Rot90::R90).0.size(), IVec2::new(5, 3));
        assert_eq!(rect.get(Rot90::R180).0.size(), IVec2::new(3, 5));
        assert_eq!(rect.get(Rot90::R270).0.size(), IVec2::new(5, 3));
    }

    #[test]
    fn right_triangle_four_corners() {
        let spec = ShapeSpec::Triangle {
            base: TriangleBase::Right,
            size: UVec2::new(3, 3),
            xform: Xform::default(),
        };
        let shape = bake(&spec);

        let mut seen = std::collections::HashSet::new();
        for rot in Rot90::ALL {
            let footprint = shape.get(rot).0;
            assert_eq!(footprint.size(), IVec2::new(3, 3));
            assert_eq!(footprint.cells().len(), 6);
            let min = footprint
                .cells()
                .iter()
                .fold(IVec2::MAX, |a, c| a.min(*c));
            assert_eq!(min, IVec2::ZERO);
            assert!(seen.insert(footprint.cells()));
        }
        assert_eq!(seen.len(), 4);
    }

    #[test]
    fn triangle_raster_matches_spec_examples() {
        let right = bake(&ShapeSpec::Triangle {
            base: TriangleBase::Right,
            size: UVec2::new(3, 3),
            xform: Xform::default(),
        });
        let widths = (0..3)
            .map(|y| {
                right
                    .get(Rot90::R0)
                    .0
                    .cells()
                    .iter()
                    .filter(|cell| cell.y == y)
                    .count()
            })
            .collect::<Vec<_>>();
        assert_eq!(widths, vec![3, 2, 1]);

        let isosceles = bake(&ShapeSpec::Triangle {
            base: TriangleBase::Isosceles,
            size: UVec2::new(5, 3),
            xform: Xform::default(),
        });
        let widths = (0..3)
            .map(|y| {
                isosceles
                    .get(Rot90::R0)
                    .0
                    .cells()
                    .iter()
                    .filter(|cell| cell.y == y)
                    .count()
            })
            .collect::<Vec<_>>();
        assert_eq!(widths, vec![5, 3, 1]);
    }

    #[test]
    fn definition_xform_mirrors_and_renormalizes() {
        let flipped = bake(&ShapeSpec::Triangle {
            base: TriangleBase::Right,
            size: UVec2::new(3, 3),
            xform: Xform {
                flip_x: true,
                ..default()
            },
        });
        let cells = flipped.get(Rot90::R0).0.cells();
        assert!(cells.contains(&IVec2::new(2, 2)));
        assert!(!cells.contains(&IVec2::new(0, 2)));

        let flipped_y = bake(&ShapeSpec::Triangle {
            base: TriangleBase::Right,
            size: UVec2::new(3, 3),
            xform: Xform {
                flip_y: true,
                ..default()
            },
        });
        let widths = (0..3)
            .map(|y| {
                flipped_y
                    .get(Rot90::R0)
                    .0
                    .cells()
                    .iter()
                    .filter(|cell| cell.y == y)
                    .count()
            })
            .collect::<Vec<_>>();
        assert_eq!(widths, vec![1, 2, 3]);

        let rotated = bake(&ShapeSpec::Rect(UVec2::new(1, 3)));
        assert_eq!(rotated.get(Rot90::R90).0.size(), IVec2::new(3, 1));
    }

    #[test]
    fn triangle_hitbox_vertices_follow_rotation() {
        let spec = ShapeSpec::Triangle {
            base: TriangleBase::Right,
            size: UVec2::new(3, 3),
            xform: Xform::default(),
        };
        let shape = bake(&spec);
        let Hitbox::Polygon(base_points) = shape.get(Rot90::R0).1 else {
            panic!("triangle must bake a polygon hitbox");
        };
        let Hitbox::Polygon(rotated_points) = shape.get(Rot90::R90).1 else {
            panic!("triangle must bake a polygon hitbox");
        };
        assert_ne!(base_points, rotated_points);
        assert!(base_points.iter().all(|p| p.x.abs() <= GRID_SIZE * 1.5 + 1e-3));
        assert!(
            rotated_points
                .iter()
                .all(|p| p.y.abs() <= GRID_SIZE * 1.5 + 1e-3)
        );
    }

    #[test]
    fn ring_mask_keeps_its_hole() {
        let shape = bake(&rows(&["###", "#.#", "###"]));
        let footprint = shape.get(Rot90::R0).0;
        assert_eq!(footprint.size(), IVec2::new(3, 3));
        assert_eq!(footprint.cells().len(), 8);
        assert!(!footprint.cells().contains(&IVec2::new(1, 1)));
        assert!(matches!(shape.get(Rot90::R0).1, Hitbox::Cells(_)));
    }

    #[test]
    fn hitboxes_build_matching_collider_shapes() {
        let cuboid = collider_from_hitbox(&Hitbox::Cuboid(Vec2::splat(32.0)));
        assert!(cuboid.as_cuboid().is_some());

        let triangle = collider_from_hitbox(&Hitbox::Polygon(vec![
            Vec2::new(-32.0, -32.0),
            Vec2::new(32.0, -32.0),
            Vec2::new(-32.0, 32.0),
        ]));
        assert!(triangle.as_convex_polygon().is_some());

        let cells = collider_from_hitbox(&Hitbox::Cells(vec![IVec2::ZERO, IVec2::X]));
        let compound = cells.as_compound().expect("compound");
        assert_eq!(compound.shapes().len(), 2);
    }

    #[test]
    fn invalid_shapes_are_rejected() {
        assert!(BakedShape::try_from(&rows(&["#..", "..#"])).is_err());
        assert!(BakedShape::try_from(&rows(&["##", "#"])).is_err());
        assert!(BakedShape::try_from(&rows(&["#x"])).is_err());
        assert!(BakedShape::try_from(&rows(&["..."])).is_err());
        assert!(BakedShape::try_from(&rows(&["...", "##.", "###"])).is_err());
        assert!(BakedShape::try_from(&ShapeSpec::Square(0)).is_err());
        assert!(BakedShape::try_from(&ShapeSpec::Rect(UVec2::new(0, 3))).is_err());

        let shrinking = ShapeSpec::Triangle {
            base: TriangleBase::Isosceles,
            size: UVec2::new(4, 3),
            xform: Xform::default(),
        };
        assert!(BakedShape::try_from(&shrinking).is_err());
    }
}
