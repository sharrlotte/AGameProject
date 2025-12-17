use bevy::prelude::*;

pub mod structures;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Home,
    Game,
    ShipEditor,
}
