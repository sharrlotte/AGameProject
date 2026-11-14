use bevy::prelude::*;
use crate::common::Dying;
use crate::config::EnemiesConfig;
use crate::world::Survivor;

use crate::enemies::ai::state::Enemy;

use super::state::{BrainState, EnemyBrain};

/// Advances aggro, alert, and stagger state from survivor proximity.
pub fn update_brains(
    time: Res<Time>,
    cfg: Res<EnemiesConfig>,
    survivor: Option<Single<(Entity, &Transform), With<Survivor>>>,
    mut brains: Query<(&mut EnemyBrain, &Transform, Has<Dying>), (With<Enemy>, Without<Survivor>)>,
) {
    let dt = time.delta_secs();
    let seen = survivor.as_ref().map(|single| {
        let (entity, transform) = **single;
        (entity, transform.translation)
    });

    for (mut brain, transform, dying) in &mut brains {
        if dying {
            continue;
        }
        let pos = transform.translation;
        let sighting = seen.map(|(entity, target_pos)| (entity, target_pos, flat_distance(target_pos, pos)));

        match brain.state {
            BrainState::Idle => {
                if let Some((entity, target_pos, dist)) = sighting
                    && dist <= cfg.aggro_radius
                {
                    brain.engage(entity, target_pos);
                }
            }
            BrainState::Engage => {
                let tracked = sighting.filter(|(_, _, dist)| *dist <= cfg.deaggro_radius);
                if let Some((entity, target_pos, _)) = tracked {
                    brain.target = Some(entity);
                    brain.last_known = Some(target_pos);
                    brain.unseen_secs = 0.0;
                } else {
                    brain.unseen_secs += dt;
                    if brain.unseen_secs >= cfg.deaggro_secs {
                        brain.state = BrainState::Alert;
                        brain.alert_timer = cfg.alert_secs;
                        brain.unseen_secs = 0.0;
                    }
                }
            }
            BrainState::Alert => {
                brain.alert_timer = (brain.alert_timer - dt).max(0.0);
                if let Some((entity, target_pos, dist)) = sighting
                    && dist <= cfg.aggro_radius
                {
                    brain.engage(entity, target_pos);
                } else if brain.alert_timer <= 0.0 {
                    brain.state = BrainState::Idle;
                    brain.target = None;
                    brain.last_known = None;
                }
            }
            BrainState::Stagger => {
                brain.stagger_timer = (brain.stagger_timer - dt).max(0.0);
                if brain.stagger_timer <= 0.0 {
                    if brain.target.is_some() {
                        brain.state = BrainState::Engage;
                    } else {
                        brain.state = BrainState::Alert;
                        brain.alert_timer = cfg.alert_secs;
                    }
                }
            }
        }
    }
}

fn flat_distance(a: Vec3, b: Vec3) -> f32 {
    Vec2::new(a.x - b.x, a.z - b.z).length()
}
