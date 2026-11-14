use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use crate::config::ConfigHotReloadedMessage;
use crate::common::{Armor, Dying};
use crate::world::Survivor;

use crate::config::EnemiesConfig;
use crate::enemies::brain::{BrainState, EnemyBrain};
use crate::enemies::detonate::Fuse;
use crate::enemies::melee::Melee;
use crate::enemies::ranged::Ranged;

use super::state::{Enemy, FacingDirection};

const RANGED_RETREAT: f32 = 0.8;
const RANGED_ADVANCE: f32 = 1.15;

pub fn freeze_enemies_on_game_over(mut enemies: Query<&mut Velocity, With<Enemy>>) {
    for mut vel in &mut enemies {
        vel.linear = Vec3::ZERO;
        vel.angular = Vec3::ZERO;
    }
}

/// Applies authored stat and behavior values to live enemies; composition changes wait for redeploy.
pub fn sync_enemies_hot_reload(
    mut reloaded: MessageReader<ConfigHotReloadedMessage>,
    cfg: Res<EnemiesConfig>,
    mut enemies: Query<(
        &mut Enemy,
        Option<&mut Melee>,
        Option<&mut Armor>,
        Option<&mut Ranged>,
        Option<&mut Fuse>,
    )>,
) {
    if reloaded.read().next().is_none() || !cfg.is_changed() {
        return;
    }

    for (mut enemy, melee, armor, ranged, fuse) in &mut enemies {
        let Some(arch) = cfg.archetype(enemy.kind) else {
            continue;
        };
        enemy.move_speed = arch.move_speed;
        enemy.mass = arch.mass;
        enemy.scrap_value = arch.scrap_value;

        match (melee, arch.melee()) {
            (Some(mut melee), Some(spec)) => melee.sync(spec),
            (None, Some(_)) => warn!(
                "enemy '{}' gained a Melee behavior; it applies on the next deploy",
                enemy.kind.label()
            ),
            (Some(_), None) => warn!(
                "enemy '{}' lost its Melee behavior; it applies on the next deploy",
                enemy.kind.label()
            ),
            (None, None) => {}
        }

        match (armor, arch.armor()) {
            (Some(mut armor), Some(spec)) => {
                armor.front_mult = spec.front_mult;
                armor.back_mult = spec.back_mult;
                armor.arc_deg = spec.arc_deg;
                armor.bypass = spec.bypass;
            }
            (None, Some(_)) => warn!(
                "enemy '{}' gained an Armor behavior; it applies on the next deploy",
                enemy.kind.label()
            ),
            (Some(_), None) => warn!(
                "enemy '{}' lost its Armor behavior; it applies on the next deploy",
                enemy.kind.label()
            ),
            (None, None) => {}
        }

        match (ranged, arch.ranged()) {
            (Some(mut ranged), Some(spec)) => ranged.sync(spec),
            (None, Some(_)) => warn!(
                "enemy '{}' gained a Ranged behavior; it applies on the next deploy",
                enemy.kind.label()
            ),
            (Some(_), None) => warn!(
                "enemy '{}' lost its Ranged behavior; it applies on the next deploy",
                enemy.kind.label()
            ),
            (None, None) => {}
        }

        match (fuse, arch.detonate()) {
            (Some(mut fuse), Some(spec)) => fuse.sync(spec),
            (None, Some(_)) => warn!(
                "enemy '{}' gained a Detonate behavior; it applies on the next deploy",
                enemy.kind.label()
            ),
            (Some(_), None) => warn!(
                "enemy '{}' lost its Detonate behavior; it applies on the next deploy",
                enemy.kind.label()
            ),
            (None, None) => {}
        }
    }
}

/// Drives horizontal velocity from brain intent, separation, and residual knockback.
#[allow(clippy::type_complexity)]
pub fn steer_enemies(
    time: Res<Time>,
    cfg: Res<EnemiesConfig>,
    survivor: Option<Single<&Transform, With<Survivor>>>,
    mut enemies: Query<
        (
            Entity,
            &mut Enemy,
            &mut Velocity,
            &Transform,
            &mut FacingDirection,
            &EnemyBrain,
            Option<&Melee>,
            Option<&Ranged>,
            Option<&Fuse>,
            Has<Dying>,
        ),
        Without<Survivor>,
    >,
) {
    let dt = time.delta_secs();
    let decay = (1.0 - cfg.knockback_decay_rate * dt).max(0.0);
    let survivor_pos = survivor.as_ref().map(|s| s.translation);

    let mut snapshot: Vec<(Entity, Vec3, f32)> = Vec::with_capacity(enemies.iter().count());
    for (entity, enemy, _, transform, _, _, _, _, _, _) in &mut enemies {
        snapshot.push((entity, transform.translation, enemy.radius));
    }

    for (entity, mut enemy, mut vel, transform, mut facing, brain, melee, ranged, fuse, dying) in
        &mut enemies
    {
        enemy.knockback_vel *= decay;
        if enemy.knockback_vel.length_squared() < 1e-4 {
            enemy.knockback_vel = Vec3::ZERO;
        }
        if dying {
            vel.linear = Vec3::ZERO;
            continue;
        }

        let pos = transform.translation;
        let mut linear = enemy.knockback_vel;
        let rooted = matches!(brain.state, BrainState::Idle | BrainState::Stagger)
            || melee.is_some_and(Melee::is_winding)
            || ranged.is_some_and(Ranged::is_winding)
            || fuse.is_some_and(Fuse::is_armed);

        if !rooted {
            let goal = match brain.state {
                BrainState::Engage => survivor_pos.or(brain.last_known),
                BrainState::Alert => brain.last_known,
                _ => None,
            };
            if let Some(goal_pos) = goal
                && let Some(dir) = flat_dir(goal_pos - pos)
            {
                facing.0 = dir;
                let travel = if brain.state == BrainState::Engage {
                    preferred_range_travel(dir, ranged, flat_distance(pos, goal_pos))
                } else {
                    dir
                };
                linear += travel * enemy.move_speed;
            }
            linear += separation(entity, pos, enemy.radius, &snapshot, &cfg);
        }

        vel.linear = linear;
    }
}

fn preferred_range_travel(dir: Vec3, ranged: Option<&Ranged>, distance: f32) -> Vec3 {
    let Some(ranged) = ranged else {
        return dir;
    };
    if distance < ranged.reach * RANGED_RETREAT {
        -dir
    } else if distance <= ranged.reach * RANGED_ADVANCE {
        Vec3::ZERO
    } else {
        dir
    }
}

fn flat_distance(a: Vec3, b: Vec3) -> f32 {
    Vec2::new(a.x - b.x, a.z - b.z).length()
}

fn flat_dir(delta: Vec3) -> Option<Vec3> {
    let flat = Vec3::new(delta.x, 0.0, delta.z);
    if flat.length_squared() <= 1e-4 {
        None
    } else {
        Some(flat.normalize())
    }
}

fn separation(
    entity: Entity,
    pos: Vec3,
    radius: f32,
    neighbors: &[(Entity, Vec3, f32)],
    cfg: &EnemiesConfig,
) -> Vec3 {
    let mut push = Vec3::ZERO;
    for (other, other_pos, other_radius) in neighbors {
        if *other == entity {
            continue;
        }
        let min_dist = radius + *other_radius;
        let delta = Vec3::new(pos.x - other_pos.x, 0.0, pos.z - other_pos.z);
        let dist = delta.length();
        if dist <= 1e-4 || dist >= cfg.separation_radius.max(min_dist) {
            continue;
        }
        let weight = 1.0 - (dist / cfg.separation_radius.max(min_dist));
        push += (delta / dist) * weight;
    }
    push.clamp_length_max(1.0) * cfg.separation_strength
}
