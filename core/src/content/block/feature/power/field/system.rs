use bevy::prelude::*;

use crate::constant::LAYER_OVERDRIVE_FIELD;

use super::{
    component::{
        Field, FieldAssets, OverdriveMaterial, OVERDRIVE_FILL_COLOR, OVERDRIVE_RING_COLOR,
        OVERDRIVE_RING_WIDTH,
    },
    lib::spawn_field_visual,
};
use crate::content::block::{
    BlockOwner, Disabled,
    feature::power::component::{Booster, BoosterState},
};
use crate::content::ship::InspectedShip;

pub fn setup_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<OverdriveMaterial>>,
) {
    let mesh = meshes.add(Rectangle::new(2.0, 2.0));
    let default_radius = 256.0;
    let ring_ratio = OVERDRIVE_RING_WIDTH / default_radius;

    let material = materials.add(OverdriveMaterial {
        color: OVERDRIVE_FILL_COLOR,
        border_color: OVERDRIVE_RING_COLOR,
        ring_ratio,
        radius: default_radius,
    });

    commands.insert_resource(FieldAssets { mesh, material });
}

pub fn spawn_field_on_booster(
    mut commands: Commands,
    boosters: Query<(Entity, &Booster), (Added<Booster>, Without<Field>)>,
    assets: Option<Res<FieldAssets>>,
    mut materials: ResMut<Assets<OverdriveMaterial>>,
) {
    let Some(assets) = assets else {
        return;
    };

    for (entity, booster) in &boosters {
        let material = if (booster.radius - 256.0).abs() < f32::EPSILON {
            assets.material.clone()
        } else {
            let ring_ratio = (OVERDRIVE_RING_WIDTH / booster.radius.max(1.0)).clamp(0.005, 0.5);
            materials.add(OverdriveMaterial {
                color: OVERDRIVE_FILL_COLOR,
                border_color: OVERDRIVE_RING_COLOR,
                ring_ratio,
                radius: booster.radius,
            })
        };

        let field = spawn_field_visual(
            &mut commands,
            entity,
            assets.mesh.clone(),
            material,
            false,
        );

        commands.entity(field).insert(Transform::from_translation(
            Vec2::ZERO.extend(LAYER_OVERDRIVE_FIELD),
        ).with_scale(Vec3::new(booster.radius, booster.radius, 1.0)));
    }
}

pub fn sync_field_visibility(
    boosters: Query<(
        &BoosterState,
        Option<&Disabled>,
        Option<&BlockOwner>,
        &Children,
    ), With<Booster>>,
    inspected: Option<Res<InspectedShip>>,
    mut fields: Query<&mut Visibility, With<Field>>,
) {
    let inspected_ship = inspected.as_ref().and_then(|i| i.0);

    for (state, disabled, owner, children) in &boosters {
        let is_selected = match (owner, inspected_ship) {
            (Some(owner), Some(inspected)) => owner.body == inspected,
            (Some(_), None) => false,
            (None, _) => true,
        };

        let is_visible = state.powered && disabled.is_none() && is_selected;
        for &child in children {
            if let Ok(mut visibility) = fields.get_mut(child) {
                let target = if is_visible {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
                if *visibility != target {
                    *visibility = target;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_field_visibility_toggles_with_power_and_disabled() {
        let mut app = App::new();
        app.add_systems(Update, sync_field_visibility);

        let field_entity = app
            .world_mut()
            .spawn((Field, Visibility::Hidden))
            .id();

        let booster_entity = app
            .world_mut()
            .spawn((
                Booster {
                    radius: 256.0,
                    boost: 1.5,
                    rate: 8.0,
                },
                BoosterState { powered: false },
            ))
            .add_child(field_entity)
            .id();

        // 1. Initially unpowered -> Hidden
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(field_entity).unwrap(),
            Visibility::Hidden
        );

        // 2. Power on -> Inherited (visible)
        app.world_mut()
            .get_mut::<BoosterState>(booster_entity)
            .unwrap()
            .powered = true;
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(field_entity).unwrap(),
            Visibility::Inherited
        );

        // 3. Add Disabled -> Hidden
        app.world_mut()
            .entity_mut(booster_entity)
            .insert(Disabled);
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(field_entity).unwrap(),
            Visibility::Hidden
        );
    }

    #[test]
    fn sync_field_visibility_respects_inspected_ship() {
        let mut app = App::new();
        app.init_resource::<InspectedShip>();
        app.add_systems(Update, sync_field_visibility);

        let ship_1 = app.world_mut().spawn_empty().id();
        let ship_2 = app.world_mut().spawn_empty().id();

        let field_entity = app
            .world_mut()
            .spawn((Field, Visibility::Hidden))
            .id();

        let _booster_entity = app
            .world_mut()
            .spawn((
                Booster {
                    radius: 256.0,
                    boost: 1.5,
                    rate: 8.0,
                },
                BoosterState { powered: true },
                BlockOwner { body: ship_1 },
            ))
            .add_child(field_entity)
            .id();

        // 1. Uninspected -> Hidden
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(field_entity).unwrap(),
            Visibility::Hidden
        );

        // 2. ship_2 inspected -> Still hidden
        app.world_mut().resource_mut::<InspectedShip>().0 = Some(ship_2);
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(field_entity).unwrap(),
            Visibility::Hidden
        );

        // 3. ship_1 inspected -> Inherited (visible)
        app.world_mut().resource_mut::<InspectedShip>().0 = Some(ship_1);
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(field_entity).unwrap(),
            Visibility::Inherited
        );

        // 4. Selection cleared -> Hidden
        app.world_mut().resource_mut::<InspectedShip>().0 = None;
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(field_entity).unwrap(),
            Visibility::Hidden
        );
    }
}
