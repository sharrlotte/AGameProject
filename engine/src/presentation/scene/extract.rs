use bevy::prelude::*;
use crate::extraction::{ExtractionZone, ScrapPickup};

use super::material::SharedVisualMaterials;

#[derive(Component)]
pub struct ExtractionPadVisual;

pub fn attach_pickup_visuals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<SharedVisualMaterials>,
    pickups: Query<(Entity, &ScrapPickup), Added<ScrapPickup>>,
) {
    for (entity, pickup) in &pickups {
        let side = pickup.radius * 1.6;
        commands.entity(entity).insert((
            Mesh3d(meshes.add(Cuboid::new(side, side, side))),
            MeshMaterial3d(materials.scrap.clone()),
        ));
    }
}

pub fn attach_extraction_visuals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<SharedVisualMaterials>,
    zones: Query<(Entity, &ExtractionZone), Added<ExtractionZone>>,
) {
    for (entity, zone) in &zones {
        commands.entity(entity).insert((
            ExtractionPadVisual,
            Mesh3d(meshes.add(Cylinder::new(zone.radius, 0.06))),
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







