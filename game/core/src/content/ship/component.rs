use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(debug_assertions)]
use crate::constant::LAYER_SHIP_UI;
use crate::config::GameConfigValue;
use crate::content::block::{BlockPlacement, BlockRegistry, spawn_block_entity};
use crate::content::ship::ai::{BehaviorIdle, BehaviorTimer};
use crate::content::ship::grid::ShipGrid;
use crate::content::ship::stat::ShipStats;
use crate::content::team::Team;
use crate::resource::{SceneState};
use crate::content::selectable::Selectable;

#[derive(Component, Debug, Default, Clone)]
pub struct Ship {
    /// Core block id.
    pub id: Uuid,
    /// Core block centre in ship-local space.
    pub core_position: Vec2,
}

#[derive(Component, Debug, Clone)]
pub struct BoundingBox {
    pub min: Vec2,
    pub max: Vec2,
    pub size: Vec2,
    pub center: Vec2,
}

impl Default for BoundingBox {
    fn default() -> Self {
        Self {
            min: Vec2::splat(f32::MAX),
            max: Vec2::splat(f32::MIN),
            size: Vec2::ZERO,
            center: Vec2::ZERO,
        }
    }
}

/// Blocks that were cut off from the core by a mutation.
#[derive(Component, Debug, Clone, Default)]
pub struct ShipFragment {
    pub placements: Vec<BlockPlacement>,
}

/// Result of deriving a ship from a grid. A missing `ship` means the layout has
/// no core and cannot be spawned as a ship.
#[derive(Clone, Debug, Default)]
pub struct ShipBuildResult {
    pub ship: Option<Ship>,
    pub stats: Option<ShipStats>,
    pub bounds: Option<BoundingBox>,
    pub fragments: Vec<ShipFragment>,
}

/// Static ship blueprint used by spawn, editor, and designs on disk.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShipDesign {
    #[serde(default = "default_version")]
    pub version: u32,
    pub id: Uuid,
    #[serde(default)]
    pub name: String,
    pub blocks: Vec<BlockPlacement>,
}

fn default_version() -> u32 {
    1
}

impl ShipDesign {
    pub fn new(blocks: Vec<BlockPlacement>) -> Self {
        Self {
            version: 1,
            id: Uuid::now_v7(),
            name: String::new(),
            blocks,
        }
    }

    pub fn spawn(
        &self,
        commands: &mut Commands,
        registry: &BlockRegistry,
        team: Team,
        transform: Transform,
        config: &GameConfigValue,
    ) -> Option<(Entity, Vec<Entity>)> {
        let mut grid = ShipGrid::new();
        grid.rebuild_from_layout(&self.blocks, registry);

        let build = registry.build_ship_result(&grid);
        let ship = build.ship?;
        let stats = build.stats?;
        let bounds = build.bounds?;
        let mass = stats.mass;

        let collider = registry.build_ship_voxel_collider_2d(&grid);

        let ship_entity = commands
            .spawn((
                ship,
                stats,
                Visibility::Visible,
                Selectable,
                DespawnOnExit(SceneState::Game),
                transform,
                team,
            ))
            .insert((
                collider,
                RigidBody::Dynamic,
                Velocity::zero(),
                ExternalForce::default(),
                ExternalImpulse::default(),
                ReadMassProperties::default(),
                Restitution::coefficient(0.005),
                CollisionGroups::new(team.ship_group(), team.ship_filter()),
                ColliderMassProperties::Mass(mass),
                ActiveEvents::COLLISION_EVENTS,
                ActiveCollisionTypes::all(),
                BehaviorTimer::new(config.0.ai_think_rate_hz),
                BehaviorIdle,
                Damping {
                    linear_damping: config.0.linear_damping,
                    angular_damping: config.0.angular_damping,
                },
                bounds,
            ))
            .id();

        let placements: Vec<BlockPlacement> = self.blocks.clone();

        let mut block_entities = Vec::new();
        for placement in placements {
            let Some(def) = registry.get_symbol(&placement.block_type) else {
                warn!("Unknown block type {:?}", placement.block_type);
                continue;
            };
            let block = spawn_block_entity(
                commands,
                &placement,
                def,
                ship_entity,
                team,
                config,
            );
            grid.set_entity(placement.id, block);
            block_entities.push(block);
        }

        commands.entity(ship_entity).insert(grid);

        #[cfg(debug_assertions)]
        {
            let ui_root = commands
                .spawn((
                    ShipUIRoot { ship: ship_entity },
                    Visibility::Visible,
                    Transform::from_translation(Vec2::ZERO.extend(LAYER_SHIP_UI)),
                ))
                .id();

            commands.entity(ship_entity).add_child(ui_root);
        }

        Some((ship_entity, block_entities))
    }
}

#[cfg(debug_assertions)]
#[derive(Component, Debug)]
pub struct ShipUIRoot {
    pub ship: Entity,
}

#[derive(Component, Debug)]
pub enum ShipBehavior {
    Approach { position: Vec2, radius: f32 },
    GoAround { position: Vec2, radius: f32 },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GameConfig;
    use crate::content::block::{BlockDef, BlockRef, BlockShape};

    fn def(block_type: &str, group: &str, mass: f32, health: f32) -> BlockDef {
        BlockDef {
            name: block_type.to_string(),
            block_type: block_type.to_string(),
            block_group: group.to_string(),
            mass,
            health,
            shape: BlockShape::default(),
            cost: 1,
            features: Vec::new(),
            drops: Vec::new(),
        }
    }

    fn spawn_design(
        mut commands: Commands,
        registry: Res<BlockRegistry>,
        config: Res<GameConfigValue>,
    ) {
        let design = ShipDesign::new(vec![
            BlockPlacement::new(IVec2::ZERO, "core", 0),
            BlockPlacement::new(IVec2::X, "plate", 0),
        ]);
        design
            .spawn(&mut commands, &registry, Team::new(0), Transform::default(), &config)
            .expect("valid design should spawn");
    }

    #[test]
    fn spawn_builds_grid_and_block_entities() {
        let mut registry = BlockRegistry::default();
        registry.register("core", def("core", "core", 10.0, 100.0));
        registry.register("plate", def("plate", "structure", 5.0, 50.0));

        let mut app = App::new();
        app.insert_resource(registry)
            .insert_resource(GameConfigValue(GameConfig::default()))
            .add_systems(Startup, spawn_design);
        app.update();

        let world = app.world_mut();
        let mut grids = world.query::<&ShipGrid>();
        let grid = grids.single(world).expect("spawned ship has a grid");
        assert_eq!(grid.blocks.len(), 2);
        assert!(grid.blocks().all(|block| block.entity != Entity::PLACEHOLDER));

        let mut refs = world.query::<&BlockRef>();
        assert_eq!(refs.iter(world).count(), 2);

        let mut stats = world.query::<&ShipStats>();
        let stats = stats.single(world).unwrap();
        assert_eq!(stats.mass, 15.0);
        assert_eq!(stats.max_health, 150.0);
    }
}
