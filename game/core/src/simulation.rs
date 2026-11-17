use bevy::prelude::*;

#[derive(SystemSet, Debug, Clone, Eq, PartialEq, Hash)]
pub enum SimulationSet {
    Input,
    Simulate,
    Apply,
    Cleanup,
}
