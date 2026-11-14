use bevy::prelude::*;

/// Cached primitive meshes for the Tank's armored assembly, authored in feet space.
#[derive(Debug, Clone)]
pub struct TankMeshes {
    pub torso: Handle<Mesh>,
    pub plate: Handle<Mesh>,
    pub weak: Handle<Mesh>,
    pub pelvis: Handle<Mesh>,
    pub head: Handle<Mesh>,
    pub shoulder: Handle<Mesh>,
    pub arm: Handle<Mesh>,
    pub leg: Handle<Mesh>,
    pub boot: Handle<Mesh>,
}

#[derive(Resource)]
pub struct SharedVisualMeshes {
    pub projectile: Handle<Mesh>,
    pub walker: Handle<Mesh>,
    pub runner: Handle<Mesh>,
    pub tank: TankMeshes,
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
            tank: TankMeshes {
                torso: meshes.add(Cuboid::new(0.66, 0.74, 0.44)),
                plate: meshes.add(Cuboid::new(0.62, 0.54, 0.10)),
                weak: meshes.add(Cuboid::new(0.50, 0.42, 0.06)),
                pelvis: meshes.add(Cuboid::new(0.52, 0.34, 0.38)),
                head: meshes.add(Cuboid::new(0.32, 0.30, 0.32)),
                shoulder: meshes.add(Cuboid::new(0.34, 0.26, 0.36)),
                arm: meshes.add(Cuboid::new(0.22, 0.78, 0.22)),
                leg: meshes.add(Cuboid::new(0.28, 0.52, 0.28)),
                boot: meshes.add(Cuboid::new(0.32, 0.24, 0.46)),
            },
            scrap: meshes.add(Cuboid::new(0.4, 0.4, 0.4)),
            extraction_pad: meshes.add(Cylinder::new(3.0, 0.06)),
        }
    }
}
