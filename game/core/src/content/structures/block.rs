use std::collections::HashSet;

use bevy::prelude::*;
use bevy_rapier2d::prelude::{Collider, ColliderMassProperties, CollisionGroups, Group};
use serde::{Deserialize, Serialize};

use crate::{
    GRID_SIZE,
    content::{
        structures::{components::Health, ship::ShipBlockHitbox},
        team::Team,
        turret::{AIFireController, PlayerFireController, TurretFeature, TurretTargeter},
    },
};

/// Represents the type of a block.
/// This acts as a key to look up static properties (Mass, Health, etc.) in the BlockRegistry.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum BlockType {
    Hull,
    Frame,
    ThrusterSmall,
    ThrusterLarge,
    ReactorSmall,
    WeaponLaser,
    Cockpit,
    Core,
    LazerTurret,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum BlockGroup {
    Core,
    Base,
    Turret,
}

/// Component attached to every block entity.
#[derive(Component, Debug, Clone, Reflect)]
pub struct Block {
    pub block_type: BlockType,
    pub rotation: u8,
    pub def: BlockDefinition,
}

/// Represents a single voxel within a block's local grid.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct Voxel {
    /// Local coordinate relative to block origin.
    pub pos: IVec2,
    /// Bitmask of faces that can connect to other blocks (Up, Down, Left, Right).
    pub connectable_faces: u8,
}

impl Default for Voxel {
    fn default() -> Self {
        Self {
            pos: IVec2::ZERO,
            connectable_faces: FACE_ALL,
        }
    }
}

/// Defines the physical shape and connectivity of a block in 2D.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct BlockShape {
    /// Bounding box dimensions (width, height)
    pub size: IVec2,
    /// List of occupied local coordinates with their connectivity info.
    pub voxels: Vec<Voxel>,
}

impl BlockShape {
    pub fn filled(width: i32, height: i32) -> Self {
        let mut voxels = Vec::with_capacity((width * height) as usize);
        for y in 0..height {
            for x in 0..width {
                voxels.push(Voxel {
                    pos: IVec2::new(x, y),
                    connectable_faces: FACE_ALL,
                });
            }
        }
        Self {
            size: IVec2::new(width, height),
            voxels,
        }
    }
}

impl Default for BlockShape {
    fn default() -> Self {
        Self {
            size: IVec2::ONE,
            voxels: vec![Voxel::default()],
        }
    }
}

/// Defines the position of a block within the ship's 2D grid.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub struct GridPosition(pub IVec2);

// Bitmasks for faces in 2D
pub const FACE_RIGHT: u8 = 0b0001; // +X
pub const FACE_LEFT: u8 = 0b0010; // -X
pub const FACE_UP: u8 = 0b0100; // +Y
pub const FACE_DOWN: u8 = 0b1000; // -Y
pub const FACE_ALL: u8 = 0b1111;

/// Enum representing distinct features/behaviors a block can have.
/// This allows us to extend block functionality without growing the base struct.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub enum BlockFeature {
    /// Provides thrust force.
    Thruster { force: f32 },
    /// Generates power.
    PowerProducer { output: f32 },
    /// Consumes power to function.
    PowerConsumer { required: f32 },
    /// Stores power.
    PowerStorage { capacity: f32 },
    /// Can deal damage.
    Weapon {
        damage: f32,
        cooldown: f32,
        range: f32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub enum LayerKind {
    Base,
    Turret,
    Engine,
    Sensor,
    Decorative,
    // keep this SMALL
}

impl Default for LayerKind {
    fn default() -> Self {
        Self::Base
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct LayerVisual {
    pub sprite: String,
    pub offset: Vec2,
    pub z: f32,
    pub animated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]

pub enum LayerFeature {
    Turret(TurretFeature),
    Recoil(RecoilFeature),
    HeatEmitter(HeatEmitterFeature),
    EnergyConsumer(EnergyConsumerFeature),
}

impl LayerFeature {
    pub fn render(&self, commands: &mut Commands, entity: Entity, team: Option<Team>) {
        match self {
            LayerFeature::Recoil(_recoil_feature) => todo!(),
            LayerFeature::HeatEmitter(_heat_emitter_feature) => todo!(),
            LayerFeature::EnergyConsumer(_energy_consumer_feature) => todo!(),
            LayerFeature::Turret(f) => {
                commands.entity(entity).insert((TurretTargeter {
                    strategy: f.strategy.clone(),
                    range: f.range,
                    turn_speed: f.turn_speed,
                    fire_all_at_once: f.fire_all_at_once,
                    barrels: f.barrels.clone(),
                    shooting_pattern: f.shooting_pattern.clone(),
                    // state
                    current_barrel_index: 0,
                    potential_targets: HashSet::default(),
                    active: false,
                    target_pos: None,
                },));

                if team.unwrap_or_default() == Team::default() {
                    commands.entity(entity).insert(PlayerFireController);
                } else {
                    commands.entity(entity).insert(AIFireController);
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct RecoilFeature {
    pub strength: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct HeatEmitterFeature {
    pub amount: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct EnergyConsumerFeature {
    pub rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct BlockLayerDefinition {
    pub id: String,

    /// e.g. "base", "turret", "engine", "radar"
    pub kind: LayerKind,

    /// Visual parts for this layer
    pub visuals: Vec<LayerVisual>,

    /// Optional feature payloads (composable)
    pub features: Vec<LayerFeature>,
}

/// A definition struct that holds the static data for a block type.
/// This allows us to define blocks once and look them up later.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct BlockDefinition {
    pub name: String,
    pub description: String,
    pub block_type: BlockType,
    pub block_group: BlockGroup,
    pub mass: f32,
    pub health: f32,
    /// Replaced `size` with `shape` to support complex voxel structures
    pub shape: BlockShape,
    pub cost: u32,
    /// A list of features this block possesses.
    pub features: Vec<BlockFeature>,
    pub layers: Vec<BlockLayerDefinition>,
}

#[derive(Component)]
pub struct PlacedBlock;

#[derive(Component)]
pub struct BlockLayer {
    pub _kind: LayerKind,
}

impl BlockDefinition {
    pub fn render(
        &self,
        commands: &mut Commands,
        asset_server: &AssetServer,
        grid_pos: IVec2,
        team: Option<Team>,
    ) -> Entity {
        let size_vec = self.shape.size.as_vec2();
        let half_size = size_vec * GRID_SIZE * 0.5;

        let anchor_pos = Vec3::new(
            grid_pos.x as f32 * GRID_SIZE,
            grid_pos.y as f32 * GRID_SIZE,
            0.0,
        );

        let pos = anchor_pos + half_size.extend(0.0);

        let entity = commands
            .spawn((
                Transform::from_translation(pos),
                Visibility::Visible,
                GridPosition(grid_pos),
                PlacedBlock,
                ShipBlockHitbox,
                Collider::cuboid(half_size.x, half_size.y),
                CollisionGroups::new(team.unwrap_or_default().get_hitbox_group(), Group::ALL),
                ColliderMassProperties::Mass(0.0),
                Block {
                    block_type: self.block_type,
                    rotation: 0,
                    def: self.clone(),
                },
                Health {
                    health: self.health,
                    max_health: self.health,
                },
                self.block_group,
                team.unwrap_or_default(),
            ))
            .id();

        for layer in &self.layers {
            let layer_entity = commands
                .spawn((
                    BlockLayer {
                        _kind: layer.kind.clone(),
                    },
                    Transform::default(),
                    GlobalTransform::default(),
                    Visibility::Inherited,
                ))
                .id();

            commands.entity(entity).add_child(layer_entity);

            // Visuals
            for visual in &layer.visuals {
                let visual_entity = commands
                    .spawn((
                        Sprite {
                            image: asset_server.load(&visual.sprite),
                            custom_size: Some(size_vec * GRID_SIZE),
                            color: Color::srgb(1.005, 1.005, 1.005),
                            ..default()
                        },
                        Transform::from_translation(visual.offset.extend(visual.z)),
                    ))
                    .id();

                commands.entity(layer_entity).add_child(visual_entity);
            }

            // Features
            for feature in &layer.features {
                feature.render(commands, layer_entity, team);
            }
        }

        return entity;
    }
}
