use bevy::prelude::*;

#[derive(Component)]
pub struct SettingRoot;

#[derive(Component)]
pub struct VolumeChange(pub f32);

#[derive(Component)]
pub struct VolumeText;

#[derive(Component)]
pub struct BloomButton;

#[derive(Component)]
pub struct BloomText;

#[derive(Component)]
pub struct CloseButton;
