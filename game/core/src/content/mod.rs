use bevy::prelude::*;

pub mod structure;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Home,
    Game,
    ShipEditor,
}
