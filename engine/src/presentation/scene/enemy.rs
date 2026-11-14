use bevy::prelude::*;
use crate::common::EnemyKind;
use crate::enemies::{Enemy, FacingDirection};

use crate::presentation::vfx::HitFlash;

use super::material::SharedVisualMaterials;
use super::mesh::SharedVisualMeshes;

#[derive(Component)]
pub struct VisualBody;

pub fn attach_enemy_visuals(
    mut commands: Commands,
    meshes: Res<SharedVisualMeshes>,
    materials: Res<SharedVisualMaterials>,
    enemies: Query<(Entity, &Enemy), Added<Enemy>>,
) {
    for (entity, enemy) in &enemies {
        let (body_mesh, base_mat) = match enemy.kind {
            EnemyKind::Walker => (meshes.walker.clone(), materials.walker.clone()),
            EnemyKind::Runner => (meshes.runner.clone(), materials.runner.clone()),
        };
        let visual_entity = commands
            .spawn((
                VisualBody,
                Mesh3d(body_mesh),
                MeshMaterial3d(base_mat.clone()),
                Transform::default(),
            ))
            .id();
        commands
            .entity(entity)
            .insert((
                Visibility::default(),
                HitFlash {
                    timer: 0.0,
                    base_material: base_mat,
                    visual_entity,
                },
            ))
            .add_child(visual_entity);
    }
}

pub fn sync_visual_facing(
    enemies: Query<(&FacingDirection, &Children), With<Enemy>>,
    mut visual_bodies: Query<&mut Transform, With<VisualBody>>,
) {
    for (facing, children) in &enemies {
        apply_facing_to_children(facing.0, children, &mut visual_bodies);
    }
}

fn apply_facing_to_children(
    dir: Vec3,
    children: &Children,
    visual_bodies: &mut Query<&mut Transform, With<VisualBody>>,
) {
    if dir.length_squared() <= 1e-4 {
        return;
    }
    let rot = Quat::from_rotation_arc(Vec3::NEG_Z, dir.normalize());
    for child in children.iter() {
        if let Ok(mut tf) = visual_bodies.get_mut(child) {
            tf.rotation = rot;
        }
    }
}







