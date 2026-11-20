use bevy::prelude::*;

use crate::constant::LAYER_OVERDRIVE_FIELD;

use super::component::{Field, OverdriveMaterial};

pub fn spawn_field_visual(
    commands: &mut Commands,
    parent: Entity,
    mesh: Handle<Mesh>,
    material: Handle<OverdriveMaterial>,
    visible: bool,
) -> Entity {
    let visibility = if visible {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };

    let field_entity = commands
        .spawn((
            Field,
            Mesh2d(mesh),
            MeshMaterial2d(material),
            Transform::from_xyz(0.0, 0.0, LAYER_OVERDRIVE_FIELD),
            visibility,
        ))
        .id();

    commands.entity(parent).add_child(field_entity);
    field_entity
}
