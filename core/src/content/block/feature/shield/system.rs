use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::constant::LAYER_BULLET;
use crate::content::block::Health;
use crate::content::block::feature::power::{STARVATION_THRESHOLD, Supply};
use crate::content::block::feature::shield::component::{
    Barrier, Shield, ShieldMaterial, ShieldShape, ShieldState,
};
use crate::content::block::placement::Disabled;
use crate::content::team::Team;

pub fn spawn_shield_barriers(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ShieldMaterial>>,
    mut query: Query<(Entity, &mut Shield, &Team), (Without<Disabled>, Added<Shield>)>,
) {
    for (emitter, mut shield, team) in query.iter_mut() {
        if shield.barrier.is_some() {
            continue;
        }

        let (collider, mesh, translation) = match shield.shape {
            ShieldShape::Circle { radius } => (
                Collider::ball(radius),
                meshes.add(Circle::new(radius)),
                Vec2::ZERO,
            ),
            ShieldShape::Plane {
                width,
                thickness,
                offset,
            } => (
                Collider::cuboid(width / 2.0, thickness / 2.0),
                meshes.add(Rectangle::new(width, thickness)),
                offset,
            ),
        };

        let material = materials.add(ShieldMaterial {
            color: team.color(1.0).to_linear(),
            hp_ratio: 1.0,
            time: 0.0,
            padding: Vec2::ZERO,
        });

        let barrier_transform = Transform::from_translation(translation.extend(LAYER_BULLET));

        let barrier = commands
            .spawn((
                Barrier {
                    emitter,
                    shape: shield.shape,
                },
                Health {
                    health: shield.max_hp,
                    max_health: shield.max_hp,
                },
                *team,
                collider,
                Sensor,
                ActiveEvents::COLLISION_EVENTS,
                CollisionGroups::new(team.block_hitbox_group(), team.block_hitbox_filter()),
                Mesh2d(mesh),
                MeshMaterial2d(material),
                barrier_transform,
                GlobalTransform::from(barrier_transform),
                Visibility::Inherited,
            ))
            .id();

        commands.entity(emitter).add_child(barrier);
        shield.barrier = Some(barrier);
    }
}

pub fn update_shield_state(
    time: Res<Time>,
    mut shields: Query<(
        Entity,
        &mut Shield,
        Option<&Supply>,
        Option<&Disabled>,
        &Team,
    )>,
    mut barriers: Query<(&mut CollisionGroups, &mut Visibility, &mut Health), With<Barrier>>,
) {
    let dt = time.delta_secs();

    for (_emitter, mut shield, supply, disabled, team) in shields.iter_mut() {
        let Some(barrier_entity) = shield.barrier else {
            continue;
        };

        let Ok((mut collision_groups, mut visibility, mut health)) =
            barriers.get_mut(barrier_entity)
        else {
            continue;
        };

        let supply_val = supply.map_or(1.0, Supply::value);
        let is_unpowered = disabled.is_some() || supply_val <= STARVATION_THRESHOLD;

        if is_unpowered {
            *collision_groups = CollisionGroups::new(Group::NONE, Group::NONE);
            *visibility = Visibility::Hidden;
            continue;
        }

        let max_hp = shield.max_hp;
        let cooldown = shield.cooldown;
        let regen_rate = shield.regen_rate;

        match shield.state {
            ShieldState::Active => {
                if health.health <= 0.0 {
                    health.health = 0.0;
                    shield.state = ShieldState::Depleted {
                        cooldown_remaining: cooldown,
                    };
                    *collision_groups = CollisionGroups::new(Group::NONE, Group::NONE);
                    *visibility = Visibility::Hidden;
                } else {
                    *collision_groups =
                        CollisionGroups::new(team.block_hitbox_group(), team.block_hitbox_filter());
                    *visibility = Visibility::Inherited;
                }
            }
            ShieldState::Depleted {
                ref mut cooldown_remaining,
            } => {
                *cooldown_remaining = (*cooldown_remaining - dt).max(0.0);
                *collision_groups = CollisionGroups::new(Group::NONE, Group::NONE);
                *visibility = Visibility::Hidden;

                if *cooldown_remaining <= 0.0 {
                    shield.state = ShieldState::Regenerating;
                }
            }
            ShieldState::Regenerating => {
                let supply_factor = supply_val.clamp(0.0, 1.0);
                health.health = (health.health + regen_rate * supply_factor * dt).min(max_hp);
                *collision_groups = CollisionGroups::new(Group::NONE, Group::NONE);
                *visibility = Visibility::Hidden;

                if health.health >= max_hp {
                    health.health = max_hp;
                    shield.state = ShieldState::Active;
                    *collision_groups =
                        CollisionGroups::new(team.block_hitbox_group(), team.block_hitbox_filter());
                    *visibility = Visibility::Inherited;
                }
            }
        }
    }
}

pub fn update_shield_visuals(
    time: Res<Time>,
    shields: Query<&Shield>,
    barriers: Query<(&Barrier, &MeshMaterial2d<ShieldMaterial>, &Health)>,
    mut materials: ResMut<Assets<ShieldMaterial>>,
) {
    let current_time = time.elapsed_secs();

    for (barrier, material_handle, health) in barriers.iter() {
        let Ok(shield) = shields.get(barrier.emitter) else {
            continue;
        };

        let Some(mut material) = materials.get_mut(&material_handle.0) else {
            continue;
        };

        material.time = current_time;
        material.hp_ratio = match shield.state {
            ShieldState::Active | ShieldState::Regenerating => {
                (health.health / health.max_health.max(1.0)).clamp(0.0, 1.0)
            }
            ShieldState::Depleted { .. } => 0.0,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shield_lifecycle_transitions_through_depleted_and_regenerating() {
        let mut app = App::new();
        app.insert_resource(Time::<()>::default());
        app.add_systems(Update, update_shield_state);

        let barrier = app
            .world_mut()
            .spawn((
                Barrier {
                    emitter: Entity::PLACEHOLDER,
                    shape: ShieldShape::Circle { radius: 100.0 },
                },
                Health {
                    health: 0.0,
                    max_health: 100.0,
                },
                CollisionGroups::default(),
                Visibility::Inherited,
            ))
            .id();

        let mut shield = Shield::new(
            ShieldShape::Circle { radius: 100.0 },
            100.0,
            2.0,
            50.0,
            None,
        );
        shield.barrier = Some(barrier);
        shield.state = ShieldState::Active;

        let emitter = app.world_mut().spawn((shield, Team::new(0))).id();
        app.world_mut()
            .entity_mut(barrier)
            .get_mut::<Barrier>()
            .expect("barrier exists")
            .emitter = emitter;

        // 1. With HP = 0, state becomes Depleted and barrier is disabled/hidden
        app.update();
        let shield_comp = app.world().get::<Shield>(emitter).expect("shield exists");
        assert!(matches!(shield_comp.state, ShieldState::Depleted { .. }));
        let vis = app.world().get::<Visibility>(barrier).expect("vis exists");
        let groups = app
            .world()
            .get::<CollisionGroups>(barrier)
            .expect("groups exist");
        assert_eq!(*vis, Visibility::Hidden);
        assert_eq!(groups.memberships, Group::NONE);

        // 2. Advance time past cooldown duration (2.0s)
        {
            let mut time = app.world_mut().resource_mut::<Time>();
            time.advance_by(std::time::Duration::from_secs_f32(2.5));
        }
        app.update();
        let shield_comp = app.world().get::<Shield>(emitter).expect("shield exists");
        assert!(matches!(shield_comp.state, ShieldState::Regenerating));

        // 3. Advance time to recharge full HP (100 HP / 50 rate = 2.0s)
        {
            let mut time = app.world_mut().resource_mut::<Time>();
            time.advance_by(std::time::Duration::from_secs_f32(2.0));
        }
        app.update();
        let shield_comp = app.world().get::<Shield>(emitter).expect("shield exists");
        assert!(matches!(shield_comp.state, ShieldState::Active));
        let health = app.world().get::<Health>(barrier).expect("health exists");
        assert!(health.health >= 100.0);
        let vis = app.world().get::<Visibility>(barrier).expect("vis exists");
        let groups = app
            .world()
            .get::<CollisionGroups>(barrier)
            .expect("groups exist");
        assert_eq!(*vis, Visibility::Inherited);
        assert_ne!(groups.memberships, Group::NONE);
    }
}
