use bevy::prelude::*;

use crate::content::{
    block::{
        base::{BlockDef, BlockOwner, ComponentOwner, ShootPattern, TargetingStrategy},
        component::{efficiency::BlockEfficiency, thruster::{RotateSpeedBase, RotateSpeedComputed}},
        radar::Radar,
        turret::{Turret, TurretState},
    },
    team::Team,
};

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

impl TurretComponent {
    pub fn spawn(
        &self,
        commands: &mut Commands,
        ship: Entity,
        base: Entity,
        def: &BlockDef,
        team: &Team,
    ) {
        commands.entity(base).insert((
            Turret {
                strategy: self.strategy.clone(),
                range: self.range,
                turn_speed: self.turn_speed,
                barrels: self.barrels.clone(),
                shooting_pattern: self.shooting_pattern.clone(),
            },
            RotateSpeedBase {
                value: self.turn_speed,
            },
            RotateSpeedComputed::default(),
            BlockEfficiency::default(),
            TurretState::default(),
            Radar::new(self.range, 1.0),
            Sprite {
                image: self.image.clone(),
                custom_size: Some(def.get_pixel_size()),
                ..default()
            },
            Transform::from_translation(self.offset.extend(self.z)),
            BlockOwner { id: ship },
            ComponentOwner { id: base },
        ));

        self.shooting_pattern.insert(commands, base, team);
    }
}
