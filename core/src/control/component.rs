use bevy::ecs::{entity::Entity, resource::Resource};

#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlMode {
    Pilot,
    #[default]
    FreeLook,
}

#[derive(Resource, Default, Debug)]
pub struct CameraTarget {
    pub following: Option<Entity>,
}

#[derive(Resource, Default, Debug)]
pub struct ControlledShip {
    pub controlling: Option<Entity>,
}
