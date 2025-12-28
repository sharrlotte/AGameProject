use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    common::entity_type::EntityType,
    constant::GRID_SIZE,
    content::{
        block::{
            radar::component::Radar,
            resource::{BlockDef, BlockType},
            turret::component::{Turret, TurretState},
        },
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

#[derive(Clone, PartialEq, Reflect)]
pub struct EnergyConsumerComponent {
    pub rate: f32,
}

#[derive(Clone, PartialEq, Reflect)]
pub struct SpriteComponent {
    pub image: Handle<Image>,
    pub offset: Vec2,
    pub z: f32,
    pub animated: bool,
}

impl BlockComponentSpawner for SpriteComponent {
    fn spawn(&self, commands: &mut Commands, base: Entity, def: &BlockDef, _team: &Team) {
        commands.entity(base).insert((
            Sprite {
                image: self.image.clone(),
                custom_size: Some(def.get_pixel_size()),
                ..default()
            },
            Transform::from_translation(self.offset.extend(self.z)),
        ));
    }
}

#[derive(Debug, Clone, Reflect, PartialEq)]
pub struct TurretComponent {
    pub strategy: TargetingStrategy,
    pub range: f32,
    pub turn_speed: f32,
    pub barrels: Vec<Vec2>,
    pub shooting_pattern: ShootPattern,
    pub image: Handle<Image>,
    pub offset: Vec2,
    pub z: f32,
    pub animated: bool,
}

impl BlockComponentSpawner for TurretComponent {
    fn spawn(&self, commands: &mut Commands, base: Entity, def: &BlockDef, team: &Team) {
        commands.entity(base).insert((
            Turret {
                strategy: self.strategy.clone(),
                range: self.range,
                turn_speed: self.turn_speed,
                barrels: self.barrels.clone(),
                shooting_pattern: self.shooting_pattern.clone(),
            },
            TurretState::default(),
            Radar::new(self.range, 1.0),
            Sprite {
                image: self.image.clone(),
                custom_size: Some(def.get_pixel_size()),
                ..default()
            },
            Transform::from_translation(self.offset.extend(self.z)),
        ));

        self.shooting_pattern.insert(commands, base, team);
    }
}

#[derive(Debug, Clone, Reflect, PartialEq)]
pub enum TargetingStrategy {
    Closest,
    MostHP,
    EnemyBullet,
}

#[derive(Debug, Clone, Reflect, PartialEq)]
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

#[derive(Debug, Clone, Reflect, PartialEq)]
pub enum ShootPattern {
    Single(SingleShootPattern),
    Burst(BurstShootPattern),
    LaserBeam(LaserBeamPattern),
    PullingBeam(PullingBeamPattern),
}

impl ShootPattern {
    pub fn insert(&self, commands: &mut Commands, base: Entity, team: &Team) {
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
                    team.clone(),
                    s.clone(),
                    LaserBeamPatternState {
                        targets: Vec::new(),
                        beam: None,
                        shooting: LaserBeamPatternShootingState::Cooldown { last_fired_at: 0.0 },
                    },
                ));
            }
            ShootPattern::PullingBeam(s) => {
                commands.entity(base).insert((
                    team.clone(),
                    s.clone(),
                    PullingBeamPatternState { beam: None },
                ));
            }
        }
    }
}

#[derive(Component, Debug, Clone, Reflect, PartialEq)]
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

#[derive(Component, Debug, Clone, Reflect, PartialEq)]
pub struct LaserBeamPattern {
    pub cooldown: f32,
    pub duration: f32,
    pub damage: f32,
    pub width: f32,
}

#[derive(Component, Debug, Clone, Reflect, PartialEq)]
pub struct PullingBeamPattern {
    pub width: f32,
}

#[derive(Component, Debug, Clone, Reflect, PartialEq)]
pub struct PullingBeamPatternState {
    pub beam: Option<Entity>,
}

#[derive(Debug, Clone, Reflect, PartialEq, Serialize, Deserialize)]
pub enum LaserBeamPatternShootingState {
    Shooting { start_time: f32 },
    Cooldown { last_fired_at: f32 },
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct LaserBeamPatternState {
    pub targets: Vec<Entity>,
    pub beam: Option<Entity>,
    pub shooting: LaserBeamPatternShootingState,
}

#[derive(Component, Clone, PartialEq)]
pub struct Block {
    pub id: Uuid,
    pub rotation: u8,
    pub position: IVec2,
    pub block_type: BlockType,
}

#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockState {
    pub block_type: BlockType,
}

#[derive(Component, Clone, PartialEq)]
pub struct ShipIdComponent {
    pub id: Entity,
}

#[derive(Component, Debug, Clone, Default, Reflect, Serialize, Deserialize)]
pub struct Health {
    pub health: f32,
    pub max_health: f32,
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct PowerConsumer {
    pub required_input: f32,
    pub active: bool,
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct PowerProducer {
    pub output: f32,
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct PowerStorage {
    pub capacity: f32,
    pub current: f32,
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct Thruster {
    pub force: f32,
    pub torque: f32,
}

#[derive(Component, Debug, Clone, Reflect, PartialEq)]
pub struct ThrusterComponent {
    pub force: f32,
    pub torque: f32,
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct Weapon {
    pub damage: f32,
    pub cooldown: f32,
    pub range: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct BlockData {
    pub id: Uuid,
    pub position: IVec2,
    pub block_type: BlockType,
    pub rotation: u8,
}

impl BlockData {
    pub fn new(&self) -> Self {
        Self {
            id: Uuid::now_v7(),
            position: self.position,
            block_type: self.block_type,
            rotation: self.rotation,
        }
    }

    pub fn position_pixel(&self) -> Vec2 {
        self.position.as_vec2() * GRID_SIZE
    }

    pub fn spawn(
        &self,
        commands: &mut Commands,
        def: &BlockDef,
        ship: Entity,
        team: Team,
    ) -> Entity {
        let half_size = def.get_pixel_size() / 2.0;
        let transform = def.to_transform(self.position);

        let child = commands
            .spawn((
                transform,
                def.block_group,
                team,
                Visibility::Visible,
                ActiveEvents::COLLISION_EVENTS,
                Collider::cuboid(half_size.x, half_size.y),
                CollisionGroups::new(team.block_hitbox_group(), team.block_hitbox_filter()),
                ColliderMassProperties::Mass(0.0),
                Sensor,
                Block {
                    id: self.id,
                    block_type: self.block_type,
                    rotation: self.rotation,
                    position: self.position,
                },
                BlockState {
                    block_type: self.block_type,
                },
                Health {
                    health: def.health,
                    max_health: def.health,
                },
                ShipIdComponent { id: ship },
                EntityType::Block,
            ))
            .id();

        commands.entity(ship).add_child(child);

        for component in &def.components {
            component.spawn(commands, child, def, &team);
        }

        return child;
    }
}

pub trait BlockComponentSpawner {
    fn spawn(&self, commands: &mut Commands, base: Entity, def: &BlockDef, team: &Team);
}

#[derive(Component, Clone, PartialEq, Reflect)]
pub enum BlockComponent {
    Sprite(SpriteComponent),
    Turret(TurretComponent),
    Recoil(RecoilComponent),
    HeatEmitter(HeatEmitterComponent),
    EnergyConsumer(EnergyConsumerComponent),
    Thruster(ThrusterComponent),
}

impl BlockComponent {
    pub fn spawn(&self, commands: &mut Commands, base: Entity, def: &BlockDef, team: &Team) {
        let component = commands
            .spawn((
                Transform::default(),
                GlobalTransform::default(),
                Visibility::Inherited,
                *team,
            ))
            .id();

        commands.entity(base).add_child(component);

        match self {
            Self::Sprite(sprite) => sprite.spawn(commands, component, def, team),
            Self::Turret(turret) => turret.spawn(commands, component, def, team),
            Self::Recoil(_) => {}
            Self::HeatEmitter(_) => {}
            Self::Thruster(_) => {}
            Self::EnergyConsumer(_) => {}
        }
    }
}
