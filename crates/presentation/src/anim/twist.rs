use bevy::prelude::*;
use data::GameState;
use world::{Health, Survivor};

use crate::anim::catalog::AnimCatalog;
use crate::anim::facing::PlayerFacing;
use crate::anim::rig::{PlayerRig, compose_world, parent_of};

const EPSILON: f32 = 1e-4;

/// Applies residual upper-body yaw twist across configured spine bones in visual-root space.
pub fn apply_upper_body_twist(
    catalog: Res<AnimCatalog>,
    state: Option<Res<State<GameState>>>,
    rigs: Query<&PlayerRig>,
    facings: Query<&PlayerFacing>,
    survivors: Query<&Health, With<Survivor>>,
    parents: Query<&ChildOf>,
    mut transforms: ParamSet<(Query<&Transform>, Query<&mut Transform>)>,
) {
    if !catalog.twist.enabled {
        return;
    }
    let game_over = state
        .as_deref()
        .map(|s| *s.get() == GameState::GameOver)
        .unwrap_or(false);

    for rig in &rigs {
        if rig.twist_bones.is_empty() {
            continue;
        }
        let Ok(health) = survivors.get(rig.survivor) else {
            continue;
        };
        if health.current <= 0.0 || game_over {
            continue;
        }
        let Ok(facing) = facings.get(rig.root) else {
            continue;
        };
        if !facing.twist_rad.is_finite() || facing.twist_rad.abs() <= EPSILON {
            continue;
        }

        for target in &rig.twist_bones {
            let angle = facing.twist_rad * target.weight;
            if angle.abs() <= EPSILON {
                continue;
            }
            let new_local = {
                let read = transforms.p0();
                let Some(bone_world) = compose_world(target.entity, rig.root, &read, &parents)
                else {
                    continue;
                };
                let Some(parent_world) = parent_of(target.entity, &parents)
                    .and_then(|parent| compose_world(parent, rig.root, &read, &parents))
                else {
                    continue;
                };
                let twisted_world = (Quat::from_rotation_y(angle) * bone_world.rotation).normalize();
                (parent_world.rotation.inverse() * twisted_world).normalize()
            };

            if !new_local.is_finite() || new_local.length_squared() <= EPSILON {
                continue;
            }
            if let Ok(mut transform) = transforms.p1().get_mut(target.entity) {
                transform.rotation = new_local;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anim::rig::{BoneSet, TwistTarget};

    fn setup_twist_world(health: f32, twist_rad: f32) -> (App, Entity, Entity) {
        let mut app = App::new();
        app.init_resource::<AnimCatalog>()
            .add_systems(Update, apply_upper_body_twist);

        let survivor = app.world_mut().spawn((Survivor, Health::new(health))).id();
        let root = app
            .world_mut()
            .spawn((
                Transform::IDENTITY,
                PlayerFacing {
                    twist_rad,
                    ..Default::default()
                },
            ))
            .id();
        let model = app
            .world_mut()
            .spawn((
                Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
                ChildOf(root),
            ))
            .id();
        let hips = app
            .world_mut()
            .spawn((
                Transform::from_rotation(Quat::from_rotation_x(0.15)),
                ChildOf(model),
            ))
            .id();
        let abdomen = app
            .world_mut()
            .spawn((
                Transform::from_rotation(Quat::from_rotation_x(-0.10)),
                ChildOf(hips),
            ))
            .id();
        let torso = app
            .world_mut()
            .spawn((
                Transform::from_rotation(Quat::from_rotation_z(0.05)),
                ChildOf(abdomen),
            ))
            .id();

        app.world_mut().spawn(PlayerRig {
            survivor,
            root,
            bones: BoneSet::default(),
            twist_bones: vec![
                TwistTarget {
                    entity: hips,
                    weight: 0.25,
                },
                TwistTarget {
                    entity: abdomen,
                    weight: 0.35,
                },
                TwistTarget {
                    entity: torso,
                    weight: 0.40,
                },
            ],
        });

        (app, root, torso)
    }

    #[test]
    fn distributes_yaw_around_root_vertical_across_pitched_spine() {
        let twist = 40.0_f32.to_radians();
        let (mut app, root, torso) = setup_twist_world(100.0, twist);

        let before = {
            let mut tf_q = app.world_mut().query::<&Transform>();
            let mut parent_q = app.world_mut().query::<&ChildOf>();
            compose_world(
                torso,
                root,
                &tf_q.query(app.world()),
                &parent_q.query(app.world()),
            )
            .expect("before torso")
        };

        app.update();

        let after = {
            let mut tf_q = app.world_mut().query::<&Transform>();
            let mut parent_q = app.world_mut().query::<&ChildOf>();
            compose_world(
                torso,
                root,
                &tf_q.query(app.world()),
                &parent_q.query(app.world()),
            )
            .expect("after torso")
        };

        let expected = (Quat::from_rotation_y(twist) * before.rotation).normalize();
        assert!(after.rotation.dot(expected).abs() > 0.9999);
    }

    #[test]
    fn dead_player_skips_procedural_spine_twist() {
        let (mut app, root, torso) = setup_twist_world(0.0, 0.5);

        let before = {
            let mut tf_q = app.world_mut().query::<&Transform>();
            let mut parent_q = app.world_mut().query::<&ChildOf>();
            compose_world(
                torso,
                root,
                &tf_q.query(app.world()),
                &parent_q.query(app.world()),
            )
            .expect("before torso")
        };

        app.update();

        let after = {
            let mut tf_q = app.world_mut().query::<&Transform>();
            let mut parent_q = app.world_mut().query::<&ChildOf>();
            compose_world(
                torso,
                root,
                &tf_q.query(app.world()),
                &parent_q.query(app.world()),
            )
            .expect("after torso")
        };

        assert!(after.rotation.dot(before.rotation).abs() > 0.9999);
    }
}
