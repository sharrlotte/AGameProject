use bevy::prelude::*;

use crate::save::component::Save;

#[derive(Component, Debug)]
#[require(Save)]
pub struct Damage {
    pub damage: f32,
}
