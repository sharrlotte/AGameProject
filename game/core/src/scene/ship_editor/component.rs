use bevy::prelude::*;

#[derive(Component)]
pub struct EditorUiRoot;

#[derive(Component)]
pub struct PlacementPreview;

#[derive(Component)]
pub struct TabButton(pub EditorTab);

#[derive(Component)]
pub struct BlockListPanel;

#[derive(Component)]
pub struct ShipListPanel;

#[derive(Component)]
pub struct LoadShipButton {
    pub filename: String,
}

#[derive(Component)]
pub struct DeployButton;

#[derive(Component)]
pub struct BackButton;

#[derive(Component)]
pub struct PlayButton;

#[derive(Component)]
pub struct ShipBase;

#[derive(Eq, Default, Clone, Copy, PartialEq, Debug)]
pub enum EditorTab {
    #[default]
    Blocks,
    Ships,
}

#[derive(Component)]
pub struct BlockPropertiesText;
