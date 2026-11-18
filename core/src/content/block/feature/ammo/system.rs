use bevy::prelude::*;

use crate::{
    constant::LAYER_BLOCK_ICON, content::block::feature::ammo::AmmoState, resource::SceneState,
};

use super::{NoAmmo, NoAmmoWarningUI, WarningIconAssets};

pub(super) fn setup_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let scale = 0.5;

    let border_size = 120.0 * scale;
    let inner_size = 100.0 * scale;

    let assets = WarningIconAssets {
        border_triangle: meshes.add(Triangle2d::new(
            Vec2::new(0.0, -border_size * 0.6),
            Vec2::new(-border_size * 0.5, border_size * 0.4),
            Vec2::new(border_size * 0.5, border_size * 0.4),
        )),

        inner_triangle: meshes.add(Triangle2d::new(
            Vec2::new(0.0, -inner_size * 0.6),
            Vec2::new(-inner_size * 0.5, inner_size * 0.4),
            Vec2::new(inner_size * 0.5, inner_size * 0.4),
        )),

        exclamation_body: meshes.add(Rectangle::new(12.0 * scale, 45.0 * scale)),
        exclamation_dot: meshes.add(Circle::new(6.0 * scale)),

        black: materials.add(Color::BLACK),
        yellow: materials.add(Color::srgb(1.0, 1.0, 0.0)),
    };

    commands.insert_resource(assets);
}

pub(super) fn update_ammo(
    mut commands: Commands,
    ammos: Query<(Entity, &AmmoState), Without<NoAmmo>>,
    assets: Res<WarningIconAssets>,
) {
    for (entity, state) in ammos {
        if !state.is_empty() {
            continue;
        }

        commands
            .spawn((
                NoAmmoWarningUI(entity),
                Transform::default(),
                Visibility::Inherited,
                DespawnOnExit(SceneState::Game),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Mesh2d(assets.border_triangle.clone()),
                    MeshMaterial2d(assets.black.clone()),
                    Transform::from_xyz(0.0, 0.0, LAYER_BLOCK_ICON),
                ));

                parent.spawn((
                    Mesh2d(assets.inner_triangle.clone()),
                    MeshMaterial2d(assets.yellow.clone()),
                    Transform::from_xyz(0.0, 0.0, LAYER_BLOCK_ICON + 1.0),
                ));

                parent.spawn((
                    Mesh2d(assets.exclamation_body.clone()),
                    MeshMaterial2d(assets.black.clone()),
                    Transform::from_xyz(0.0, 5.0, LAYER_BLOCK_ICON + 2.0),
                ));

                parent.spawn((
                    Mesh2d(assets.exclamation_dot.clone()),
                    MeshMaterial2d(assets.black.clone()),
                    Transform::from_xyz(0.0, -10.0, LAYER_BLOCK_ICON + 2.0),
                ));
            });

        commands.entity(entity).insert(NoAmmo);
    }
}

pub(super) fn update_no_ammo_warning(
    mut commands: Commands,
    mut uis: Query<(Entity, &NoAmmoWarningUI)>,
    ammos: Query<(Entity, &AmmoState, &GlobalTransform)>,
) {
    for (entity, ui) in &mut uis {
        if let Ok((parent, state, global_transform)) = ammos.get(ui.0) {
            if state.is_empty() {
                commands
                    .entity(entity)
                    .insert(Transform::from_translation(global_transform.translation()));
            } else {
                commands.entity(parent).remove::<NoAmmo>();
                commands.entity(entity).despawn();
            }
        } else {
            commands.entity(entity).despawn();
        }
    }
}
