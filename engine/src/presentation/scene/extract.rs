use bevy::prelude::*;
use crate::extraction::{ExtractionZone, ScrapPickup};

use super::material::SharedVisualMaterials;
use super::mesh::SharedVisualMeshes;

#[derive(Component)]
pub struct ExtractionPadVisual;

pub fn attach_pickup_visuals(
    mut commands: Commands,
    meshes: Res<SharedVisualMeshes>,
    materials: Res<SharedVisualMaterials>,
    pickups: Query<Entity, Added<ScrapPickup>>,
) {
    for entity in &pickups {
        commands.entity(entity).insert((
            Mesh3d(meshes.scrap.clone()),
            MeshMaterial3d(materials.scrap.clone()),
        ));
    }
}

pub fn attach_extraction_visuals(
    mut commands: Commands,
    meshes: Res<SharedVisualMeshes>,
    materials: Res<SharedVisualMaterials>,
    zones: Query<Entity, Added<ExtractionZone>>,
) {
    for entity in &zones {
        commands.entity(entity).insert((
            ExtractionPadVisual,
            Mesh3d(meshes.extraction_pad.clone()),
            MeshMaterial3d(materials.extraction_idle.clone()),
        ));
    }
}

pub fn update_extraction_pad_visual(
    materials: Res<SharedVisualMaterials>,
    mut pads: Query<
        (&ExtractionZone, &mut MeshMaterial3d<StandardMaterial>),
        With<ExtractionPadVisual>,
    >,
) {
    for (zone, mut mat) in &mut pads {
        mat.0 = if zone.active {
            materials.extraction_active.clone()
        } else {
            materials.extraction_idle.clone()
        };
    }
}

pub fn animate_scrap_pickups(time: Res<Time>, mut pickups: Query<&mut Transform, With<ScrapPickup>>) {
    let dt = time.delta_secs();
    for mut tf in &mut pickups {
        tf.rotate_y(2.4 * dt);
    }
}







