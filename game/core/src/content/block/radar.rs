use bevy::prelude::*;
use bevy_rapier2d::{parry::shape::Ball, prelude::*};

use crate::content::{ship::base::Ship, team::Team};

#[derive(Component)]
pub struct Radar {
    pub range: f32,
    pub timer: Timer,
    pub targets: Vec<Entity>,
}

impl Radar {
    pub fn new(range: f32, interval: f32) -> Self {
        Self {
            range,
            timer: Timer::from_seconds(interval, TimerMode::Repeating),
            targets: Vec::new(),
        }
    }
}

pub fn radar_scan(
    time: Res<Time>,
    rapier: ReadRapierContext,
    mut radars: Query<(Entity, &mut Radar, &Team, &GlobalTransform)>,
    ships: Query<&Team, With<Ship>>,
) {
    for (_radar_entity, mut radar, team, radar_tf) in radars.iter_mut() {
        radar.timer.tick(time.delta());

        if !radar.timer.just_finished() {
            continue;
        }

        radar.targets.clear();

        let origin = radar_tf.translation().truncate();

        rapier.single().unwrap().intersect_shape(
            origin,
            Rot::default(),
            &Ball::new(radar.range),
            QueryFilter::from(CollisionGroups::new(
                team.radar_group(),
                team.radar_filter(),
            )),
            |entity| {
                if let Ok(ship_team) = ships.get(entity) {
                    if ship_team.id != team.id {
                        radar.targets.push(entity);
                    }
                }
                true
            },
        );
    }
}
