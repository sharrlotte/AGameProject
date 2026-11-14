use bevy::prelude::*;

use crate::common::{GameScene, RunEntity};
use crate::enemies::ranged::AcidProjectile;
use crate::hazard::HazardMarker;
use crate::message::messages::DetonationMessage;

use super::material::SharedVisualMaterials;
use super::mesh::SharedVisualMeshes;

const BLAST_FLASH_SECS: f32 = 0.25;

#[derive(Component)]
pub struct BlastFlash {
    timer: f32,
    base: Vec3,
}

pub fn attach_hazard_visuals(
    mut commands: Commands,
    meshes: Res<SharedVisualMeshes>,
    materials: Res<SharedVisualMaterials>,
    markers: Query<Entity, Added<HazardMarker>>,
) {
    for entity in &markers {
        commands.entity(entity).insert((
            Mesh3d(meshes.hazard.clone()),
            MeshMaterial3d(materials.acid.clone()),
        ));
    }
}

pub fn attach_acid_visuals(
    mut commands: Commands,
    meshes: Res<SharedVisualMeshes>,
    materials: Res<SharedVisualMaterials>,
    projectiles: Query<Entity, Added<AcidProjectile>>,
) {
    for entity in &projectiles {
        commands.entity(entity).insert((
            Mesh3d(meshes.acid.clone()),
            MeshMaterial3d(materials.acid.clone()),
        ));
    }
}

pub fn spawn_blast_flashes(
    mut commands: Commands,
    meshes: Res<SharedVisualMeshes>,
    materials: Res<SharedVisualMaterials>,
    mut blasts: MessageReader<DetonationMessage>,
) {
    for blast in blasts.read() {
        let radius = blast.radius.max(0.1);
        let base = Vec3::new(radius, radius * 0.4, radius);
        commands.spawn((
            Name::new("BlastFlash"),
            RunEntity,
            DespawnOnExit(GameScene::Arena),
            BlastFlash {
                timer: BLAST_FLASH_SECS,
                base,
            },
            Mesh3d(meshes.blast.clone()),
            MeshMaterial3d(materials.blast.clone()),
            Transform::from_xyz(blast.position.x, 0.25, blast.position.z).with_scale(base),
            Visibility::default(),
        ));
    }
}

pub fn animate_blast_flashes(
    time: Res<Time>,
    mut commands: Commands,
    mut flashes: Query<(Entity, &mut BlastFlash, &mut Transform)>,
) {
    let dt = time.delta_secs();
    for (entity, mut flash, mut transform) in &mut flashes {
        flash.timer -= dt;
        if flash.timer <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        let factor = (flash.timer / BLAST_FLASH_SECS).clamp(0.05, 1.0);
        transform.scale = flash.base * factor;
    }
}
