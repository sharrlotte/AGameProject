use bevy::prelude::*;
use std::path::PathBuf;

#[derive(Message)]
pub struct SaveWorldEvent {
    pub path: PathBuf,
}

#[derive(Message)]
pub struct LoadWorldEvent {
    pub path: PathBuf,
}
