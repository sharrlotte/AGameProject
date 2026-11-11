use bevy::prelude::*;
use data::EnemyKind;
use enemies::{Enemy, FacingDirection};

use crate::vfx::HitFlash;

use super::material::SharedVisualMaterials;

#[derive(Component)]
pub struct VisualBody;

pub fn attach_enemy_visuals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<SharedVisualMaterials>,
    enemies: Query<(Entity, &Enemy), Added<Enemy>>,
) {
    for (entity, enemy) in &enemies {
        let body_mesh = meshes.add(Capsule3d::new(enemy.radius, enemy.half_height * 2.0));
        let base_mat = match enemy.kind {
            EnemyKind::Walker => materials.walker.clone(),
            EnemyKind::Runner => materials.runner.clone(),
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
