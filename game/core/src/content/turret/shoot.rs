use crate::content::bullet::spawner::spawn_bullet;
use crate::content::team::Team;
use crate::content::turret::{ShootPattern, TurretTargeter};
use bevy::prelude::*;

pub fn handle_turret_emission(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(&GlobalTransform, &mut TurretTargeter, Option<&Team>)>,
) {
    for (transform, mut targeter, team) in query.iter_mut() {
        if !targeter.active {
            continue;
        }

        let TurretTargeter {
            shooting_pattern,
            barrels,
            range,
            ..
        } = &mut *targeter;

        match shooting_pattern {
            ShootPattern::Single {
                fire_rate,
                last_fired_at,
                projectile,
            } => {
                if time.elapsed_secs() - *last_fired_at >= *fire_rate {
                    for barrel_pos in barrels.iter_mut() {
                        spawn_bullet(
                            &mut commands,
                            transform,
                            team,
                            *barrel_pos,
                            *range,
                            &projectile,
                        );
                    }
                    *last_fired_at = time.elapsed_secs();
                }
            }
            ShootPattern::Burst {
                count,
                delay,
                current_shot,
                burst_timer,
                is_bursting,
                projectile,
            } => {}
            ShootPattern::Continuous => {}
        }
    }
}
