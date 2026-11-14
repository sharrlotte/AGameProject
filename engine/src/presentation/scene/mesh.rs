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

#[derive(Debug, Clone)]
pub struct SpitterMeshes {
    pub body: Handle<Mesh>,
    pub sac: Handle<Mesh>,
    pub jaw: Handle<Mesh>,
    pub arm: Handle<Mesh>,
    pub leg: Handle<Mesh>,
}

#[derive(Debug, Clone)]
pub struct ExploderMeshes {
    pub body: Handle<Mesh>,
    pub pustule: Handle<Mesh>,
    pub head: Handle<Mesh>,
    pub limb: Handle<Mesh>,
}

#[derive(Resource)]
pub struct SharedVisualMeshes {
    pub projectile: Handle<Mesh>,
    pub walker: Handle<Mesh>,
    pub runner: Handle<Mesh>,
    pub tank: TankMeshes,
    pub spitter: SpitterMeshes,
    pub exploder: ExploderMeshes,
    pub acid: Handle<Mesh>,
    pub hazard: Handle<Mesh>,
    pub blast: Handle<Mesh>,
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
            spitter: SpitterMeshes {
                body: meshes.add(Capsule3d::new(0.34, 0.62)),
                sac: meshes.add(Sphere::new(0.16)),
                jaw: meshes.add(Cuboid::new(0.26, 0.14, 0.24)),
                arm: meshes.add(Cuboid::new(0.10, 0.52, 0.10)),
                leg: meshes.add(Cuboid::new(0.13, 0.44, 0.13)),
            },
            exploder: ExploderMeshes {
                body: meshes.add(Sphere::new(0.50)),
                pustule: meshes.add(Sphere::new(0.14)),
                head: meshes.add(Sphere::new(0.22)),
                limb: meshes.add(Cuboid::new(0.16, 0.42, 0.16)),
            },
            acid: meshes.add(Sphere::new(0.12)),
            hazard: meshes.add(Cylinder::new(1.0, 0.04)),
            blast: meshes.add(Sphere::new(1.0)),
            scrap: meshes.add(Cuboid::new(0.4, 0.4, 0.4)),
            extraction_pad: meshes.add(Cylinder::new(3.0, 0.06)),
        }
    }
}
