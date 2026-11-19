use bevy::prelude::*;

#[derive(Resource, Default, Debug)]
pub struct InspectorBuildState {
    pub ship: Option<Entity>,
}

#[derive(Component)]
pub struct ShipInspectorRoot;

#[derive(Component)]
pub struct InspectorBlockGrid;

#[derive(Component)]
pub struct InspectorBlockNode(pub Entity);

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum InspectorText {
    Header,
    Hp,
    Power,
    Stored,
    Ammo,
    Stats,
    #[cfg(feature = "dev")]
    Dev,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum InspectorBar {
    Hp,
    Power,
    Stored,
    Ammo,
}
