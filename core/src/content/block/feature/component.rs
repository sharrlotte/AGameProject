use bevy::prelude::*;

use crate::config::GameConfigValue;
use crate::content::{
    block::{
        feature::{
            power::{Battery, Booster, Consumer, Generator},
            sprite::Texture,
            thruster::Thruster,
            turret::TurretComponent,
        },
    },
    team::Team,
};

#[derive(Component, Clone, PartialEq, Reflect)]
pub enum Feature {
    Texture(Texture),
    Turret(TurretComponent),
    Generator(Generator),
    Battery(Battery),
    Consumer(Consumer),
    Booster(Booster),
    Thruster(Thruster),
    Storage(u32),
}

impl Feature {
    pub fn insert(
        &self,
        commands: &mut Commands,
        base: Entity,
        visual: Entity,
        pixel_size: Vec2,
        _team: &Team,
        config: &GameConfigValue,
    ) {
        match self {
            Self::Texture(sprite) => sprite.insert(commands, visual, pixel_size),
            Self::Turret(turret) => turret.insert(commands, base, visual, pixel_size),
            Self::Thruster(thruster) => thruster.insert(commands, base, config),
            Self::Generator(generator) => generator.insert(commands, base),
            Self::Battery(battery) => battery.insert(commands, base),
            Self::Consumer(consumer) => consumer.insert(commands, base),
            Self::Booster(booster) => booster.insert(commands, base),
            Self::Storage(_) => {
            }
        }
    }

    pub fn extra_texture(&self) -> Option<&Texture> {
        match self {
            Self::Texture(t) => Some(t),
            _ => None,
        }
    }
}
