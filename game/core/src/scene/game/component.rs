use bevy::prelude::*;

#[derive(Component)]
pub struct FpsText;

#[derive(Component)]
pub struct TurretControlModeButton;

#[derive(Component)]
pub struct TurretControlModeText;

#[derive(Component)]
pub struct ShipButton(pub Entity);

#[derive(Component)]
pub struct ShipList;


#[derive(Component)]
pub struct BlockButton(pub Entity);
