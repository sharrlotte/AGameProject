use super::Radar;
use crate::content::{block::Disabled, ship::Ship, team::Team};
use bevy::prelude::*;
use bevy_rapier2d::{parry::shape::Ball, prelude::*};


pub fn radar_scan(
    time: Res<Time>,
    rapier: ReadRapierContext,
    mut radars: Query<(Entity, &mut Radar, &Team, &GlobalTransform), Without<Disabled>>,
    ships: Query<&Team, With<Ship>>,
) {
    let Ok(collision) = rapier.single() else {
        return;
    };

    for (_radar_entity, mut radar, team, radar_tf) in radars.iter_mut() {
        radar.timer.tick(time.delta());

        if !radar.timer.just_finished() {
            continue;
        }

        radar.targets.clear();

        let origin = radar_tf.translation().truncate();
        let shape = &Ball::new(radar.range);
        let filter = QueryFilter::from(CollisionGroups::new(
            team.radar_group(),
            team.radar_filter(),
        ));

        let should_continue = true;

        collision.intersect_shape(
            origin,
            Rot::default(),
            shape,
            filter,
            |entity, _collider| {
                if let Ok(ship_team) = ships.get(entity) {
                    if ship_team.id() != team.id() {
                        radar.targets.push(entity);
                    }
                }
                should_continue
            },
        );
    }
}

pub fn reset_disabled_radar(
    trigger: On<Add<Disabled>>,
    mut radars: Query<&mut Radar>,
) {
    if let Ok(mut radar) = radars.get_mut(trigger.event().entity) {
        radar.targets.clear();
    }
}
