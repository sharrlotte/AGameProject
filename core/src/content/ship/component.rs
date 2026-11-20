use bevy::prelude::*;

use bevy_rapier2d::prelude::*;

use serde::{Deserialize, Serialize};

use uuid::Uuid;


use crate::content::ship::inspect::on_ship_click;

use crate::config::GameConfigValue;

use crate::content::block::{BlockPlacement, BlockRegistry, spawn_block_entity};

use crate::content::selectable::Selectable;

use crate::content::ship::ai::{Brain, Doctrine, FireOrder, Readiness, Sensor, Skill, Steering};

use crate::content::body::BlockGrid;

use crate::content::ship::stat::ShipStats;

use crate::content::ship::throttle::Throttle;

use crate::content::team::Team;

use crate::resource::SceneState;
use crate::save::design::DESIGN_SCHEMA_VERSION;


#[derive(Component, Debug, Default, Clone)]
#[require(Throttle)]
pub struct Ship {
    pub id: Uuid,
    pub core_id: Uuid,
    pub core_position: Vec2,
}


#[derive(Component, Debug, Clone, Default)]
pub struct BoundingBox {
    min: Vec2,
    max: Vec2,
    center: Vec2,
    size: Vec2,
}

impl BoundingBox {
    pub fn from_min_max(min: Vec2, max: Vec2) -> Self {
        Self {
            min,
            max,
            center: (min + max) * 0.5,
            size: max - min,
        }
    }

    pub fn min(&self) -> Vec2 { self.min }
    pub fn max(&self) -> Vec2 { self.max }
    pub fn center(&self) -> Vec2 { self.center }
    pub fn size(&self) -> Vec2 { self.size }

    pub fn corners(&self) -> [Vec2; 4] {
        [
            Vec2::new(self.min.x, self.min.y),
            Vec2::new(self.max.x, self.min.y),
            Vec2::new(self.max.x, self.max.y),
            Vec2::new(self.min.x, self.max.y),
        ]
    }
}


#[derive(Clone, Debug)]
pub struct GridBuildResult {
    pub ship: Ship,
    pub stats: ShipStats,
    pub bounds: BoundingBox,
}


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ShipDesign {
    #[serde(default = "default_version")]
    pub version: u32,
    pub id: Uuid,
    #[serde(default)]
    pub name: String,
    pub blocks: Vec<BlockPlacement>,
    #[serde(default)]
    pub doctrine: Doctrine,
}


impl ShipDesign {
    pub fn validate(&self, registry: &BlockRegistry) -> Result<(), Vec<String>> {
        registry.validate_ship(&self.blocks)
    }

    #[cfg(test)]
    pub fn new(blocks: Vec<BlockPlacement>) -> Self {
        Self {
            version: DESIGN_SCHEMA_VERSION,
            id: Uuid::now_v7(),
            name: String::new(),
            blocks,
            doctrine: Doctrine::default(),
        }
    }

    pub fn spawn(
        &self,
        commands: &mut Commands,
        registry: &BlockRegistry,
        team: Team,
        transform: Transform,
        config: &GameConfigValue,
    ) -> Result<(Entity, Vec<Entity>), Vec<String>> {
        self.spawn_with_id(
            commands,
            registry,
            team,
            transform,
            config,
            Uuid::now_v7(),
        )
    }

    pub fn spawn_with_id(
        &self,
        commands: &mut Commands,
        registry: &BlockRegistry,
        team: Team,
        transform: Transform,
        config: &GameConfigValue,
        ship_id: Uuid,
    ) -> Result<(Entity, Vec<Entity>), Vec<String>> {
        registry.validate_ship(&self.blocks)?;

        let mut grid = BlockGrid::new();
        grid.rebuild_from_layout(&self.blocks, registry);

        let build = registry
            .build_grid_result(&grid)
            .ok_or_else(|| vec!["ship layout has no core".to_string()])?;
            
        let mut ship = build.ship;
        ship.id = ship_id;
        let stats = build.stats;
        let bounds = build.bounds;
        let mass = stats.mass;

        let collider = registry.build_voxel_collider_2d(&grid);

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
            ))
            .insert((
                self.doctrine,
                Skill::default(),
                Sensor::default(),
                Readiness::default(),
                Brain::new(transform.translation.truncate(), 5.0),
                Steering::default(),
                FireOrder::default(),
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
            let block = spawn_block_entity(commands, &placement, def, ship_entity, team, config);
            grid.set_entity(placement.id, block);
            block_entities.push(block);
        }

        commands.entity(ship_entity).insert(grid);
        commands.entity(ship_entity).observe(on_ship_click);

        Ok((ship_entity, block_entities))
    }
}


fn default_version() -> u32 {
    DESIGN_SCHEMA_VERSION
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GameConfig;
    use crate::content::block::{BakedShape, BlockDef, BlockRef, Rot90, ShapeSpec};

    fn def(block_type: &str, group: &str, mass: f32, health: f32) -> BlockDef {
        BlockDef {
            name: block_type.to_string(),
            block_type: block_type.to_string(),
            block_group: group.to_string(),
            mass,
            health,
            shape: BakedShape::try_from(&ShapeSpec::Square(1)).expect("test shape bakes"),
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
            BlockPlacement::new(IVec2::ZERO, "core", Rot90::R0),
            BlockPlacement::new(IVec2::X, "plate", Rot90::R0),
        ]);
        design
            .spawn(
                &mut commands,
                &registry,
                Team::new(0),
                Transform::default(),
                &config,
            )
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
        let mut grids = world.query::<&BlockGrid>();
        let grid = grids.single(world).expect("spawned ship has a grid");
        assert_eq!(grid.block_count(), 2);
        assert!(
            grid.blocks()
                .all(|block| block.entity != Entity::PLACEHOLDER)
        );

        let mut refs = world.query::<&BlockRef>();
        assert_eq!(refs.iter(world).count(), 2);

        let mut stats = world.query::<&ShipStats>();
        let stats = stats.single(world).unwrap();
        assert_eq!(stats.mass, 15.0);
        assert_eq!(stats.max_health, 150.0);
    }

    #[test]
    fn bounding_box_from_min_max_sets_center_size_and_corners() {
        let bounds = BoundingBox::from_min_max(Vec2::new(-16.0, 32.0), Vec2::new(48.0, 96.0));
        assert_eq!(bounds.min, Vec2::new(-16.0, 32.0));
        assert_eq!(bounds.max, Vec2::new(48.0, 96.0));
        assert_eq!(bounds.center(), Vec2::new(16.0, 64.0));
        assert_eq!(bounds.size(), Vec2::new(64.0, 64.0));
        assert_eq!(
            bounds.corners(),
            [
                Vec2::new(-16.0, 32.0),
                Vec2::new(48.0, 32.0),
                Vec2::new(48.0, 96.0),
                Vec2::new(-16.0, 96.0),
            ]
        );
    }
}
