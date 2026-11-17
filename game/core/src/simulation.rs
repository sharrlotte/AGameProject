use bevy::prelude::*;

#[derive(SystemSet, Debug, Clone, Eq, PartialEq, Hash)]
pub enum SimulationSet {
    Input,
    Stats,
    Ai,
    Control,
    Weapons,
    Damage,
    Death,
    Present,
    Cleanup,
}
