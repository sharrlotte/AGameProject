use bevy::prelude::*;

#[derive(Resource)]
pub struct SharedVisualMeshes {
    pub projectile: Handle<Mesh>,
    pub walker: Handle<Mesh>,
    pub runner: Handle<Mesh>,
    pub scrap: Handle<Mesh>,
    pub extraction_pad: Handle<Mesh>,
}

impl FromWorld for SharedVisualMeshes {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        Self {
            projectile: meshes.add(Sphere::new(0.08)),
            walker: meshes.add(Capsule3d::new(0.45, 0.9)),
            runner: meshes.add(Capsule3d::new(0.35, 0.7)),
            scrap: meshes.add(Cuboid::new(0.4, 0.4, 0.4)),
            extraction_pad: meshes.add(Cylinder::new(3.0, 0.06)),
        }
    }
}
