use super::Radar;
use crate::content::{ship::Ship, team::Team};
use crate::simulation::SimulationSet;
use bevy::prelude::*;
use bevy_rapier2d::{parry::shape::Ball, prelude::*};

pub struct RadarPlugin;

impl Plugin for RadarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, radar_scan.in_set(SimulationSet::Weapons));
    }
}

pub fn radar_scan(
    time: Res<Time>,
    rapier: ReadRapierContext,
    mut radars: Query<(Entity, &mut Radar, &Team, &GlobalTransform)>,
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
                    if ship_team.id != team.id {
                        radar.targets.push(entity);
                    }
                }
                should_continue
            },
        );
    }
}
