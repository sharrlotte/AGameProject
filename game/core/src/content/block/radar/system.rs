use bevy::prelude::*;
use bevy_rapier2d::{parry::shape::Ball, prelude::*};

use crate::content::{block::radar::component::Radar, ship::component::Ship, team::Team};

pub fn radar_scan(
    time: Res<Time>,
    rapier: ReadRapierContext,
    mut radars: Query<(Entity, &mut Radar, &Team, &GlobalTransform)>,
    ships: Query<&Team, With<Ship>>,
) {
    let collision: RapierContext<'_> = rapier.single().expect("How to get rapier context?");

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

        collision.intersect_shape(origin, Rot::default(), shape, filter, |entity| {
            if let Ok(ship_team) = ships.get(entity) {
                if ship_team.id != team.id {
                    radar.targets.push(entity);
                }
            }
            should_continue
        });
    }
}
