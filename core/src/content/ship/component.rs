use bevy::prelude::*;

use bevy_rapier2d::prelude::*;

use serde::{Deserialize, Serialize};

use uuid::Uuid;


#[cfg(feature = "dev")]
use {
    crate::constant::LAYER_SHIP_UI,
    crate::content::ship::debug::{ShipUIRoot, on_ship_click},
};

use crate::config::GameConfigValue;

use crate::content::block::{BlockPlacement, BlockRegistry, spawn_block_entity};

use crate::content::selectable::Selectable;

use crate::content::ship::ai::{AiProfile, AiRuntime};

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
    pub size: Vec2,
}


#[derive(Clone, Debug)]
pub struct GridBuildResult {
    pub ship: Ship,
    pub stats: ShipStats,
    pub bounds: BoundingBox,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShipDesign {
    #[serde(default = "default_version")]
    pub version: u32,
    pub id: Uuid,
    #[serde(default)]
    pub name: String,
    pub blocks: Vec<BlockPlacement>,
    #[serde(default)]
    pub ai: AiProfile,
}


impl ShipDesign {
    #[cfg(test)]
    pub fn new(blocks: Vec<BlockPlacement>) -> Self {
        Self {
            version: DESIGN_SCHEMA_VERSION,
            id: Uuid::now_v7(),
            name: String::new(),
            blocks,
            ai: AiProfile::default(),
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
    ) -> Option<(Entity, Vec<Entity>)> {
        let mut grid = BlockGrid::new();
        
        grid.rebuild_from_layout(&self.blocks, registry);

        let build = registry.build_grid_result(&grid)?;
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
                self.ai.clone(),
                AiRuntime::new(&self.ai, transform.translation.truncate()),
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

        #[cfg(feature = "dev")]
        {
            let ui_root = commands
                .spawn((
                    ShipUIRoot { ship: ship_entity },
                    Visibility::Hidden,
                    Transform::from_translation(Vec2::ZERO.extend(LAYER_SHIP_UI)),
                    Text2d::new(String::new()),
                    TextFont {
                        font_size: FontSize::Px(30.0),
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .id();

            commands.entity(ship_entity).add_child(ui_root);
            commands.entity(ship_entity).observe(on_ship_click);
        }

        Some((ship_entity, block_entities))
    }
}


#[derive(Component, Debug)]
pub enum ShipBehavior {
    Approach { position: Vec2, radius: f32 },
    GoAround { position: Vec2, radius: f32 },
    AimAt { target: Vec2, hold_distance: f32, aim_offset_angle: f32 },
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
        assert_eq!(grid.blocks.len(), 2);
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
}
