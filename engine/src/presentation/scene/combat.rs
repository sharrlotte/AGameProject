use bevy::prelude::*;
use crate::combat::Projectile;

use super::material::SharedVisualMaterials;

pub fn attach_projectile_visuals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<SharedVisualMaterials>,
    projectiles: Query<(Entity, &Projectile), Added<Projectile>>,
) {
    for (entity, proj) in &projectiles {
        let sphere_mesh = meshes.add(Sphere::new(proj.radius));
        commands.entity(entity).insert((
            Mesh3d(sphere_mesh),
            MeshMaterial3d(materials.projectile.clone()),
        ));
    }
}







