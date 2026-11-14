use bevy::prelude::*;
use crate::combat::Projectile;

use super::material::SharedVisualMaterials;
use super::mesh::SharedVisualMeshes;

pub fn attach_projectile_visuals(
    mut commands: Commands,
    meshes: Res<SharedVisualMeshes>,
    materials: Res<SharedVisualMaterials>,
    projectiles: Query<Entity, Added<Projectile>>,
) {
    for entity in &projectiles {
        commands.entity(entity).insert((
            Mesh3d(meshes.projectile.clone()),
            MeshMaterial3d(materials.projectile.clone()),
        ));
    }
}

