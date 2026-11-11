use bevy::animation::RepeatAnimation;
use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;
use combat::{CombatStance, WeaponInventory};
use data::GameState;
use world::{AimDirection, Health, Stamina, Survivor};

use crate::anim::blend::{self, Weights};
use crate::anim::catalog::AnimCatalog;
use crate::anim::graph::{ClipNodes, GraphNodes};
use crate::anim::{PlayerAnimAssets, PlayerRig, stance};
use crate::config::VisualsConfig;

const LOCOMOTION_ROLES: [&str; 6] = [
    "idle",
    "walk_fwd",
    "walk_back",
    "sprint",
    "strafe_l",
    "strafe_r",
];

/// Per-player animation state that must survive across frames.
#[derive(Component, Debug, Clone, Default)]
pub struct DirectorState {
    pub aim: f32,
    pub reload: f32,
    pub reload_active: bool,
    pub dead: bool,
    pub idle: f32,
    pub sprint: f32,
    pub locomotion: Weights,
}

/// Plays all looped locomotion clips at zero weight once, so the director can
/// move weights without restarting clips mid-gameplay.
pub fn prime_looped(player: &mut AnimationPlayer, nodes: &GraphNodes) {
    for role in LOCOMOTION_ROLES {
        let Some(clip) = nodes.get(role) else {
            continue;
        };
        for node in [clip.lower_node(), clip.upper_node()].into_iter().flatten() {
            let active = player.play(node);
            active.repeat();
            active.set_weight(0.0);
            active.set_speed(clip.speed.max(0.01));
        }
    }
}

/// Drives layer weights, clip playback speed, reload seeking, and death.
#[allow(clippy::too_many_arguments)]
pub fn drive_player_animation(
    time: Res<Time>,
    state: Res<State<GameState>>,
    catalog: Res<AnimCatalog>,
    visuals: Res<VisualsConfig>,
    assets: Res<PlayerAnimAssets>,
    clips: Res<Assets<AnimationClip>>,
    survivors: Query<(&Health, &Stamina, &Velocity, &AimDirection), With<Survivor>>,
    stances: Query<&CombatStance>,
    inventories: Query<&WeaponInventory>,
    mut players: Query<(&PlayerRig, &mut AnimationPlayer, &mut DirectorState)>,
) {
    let Some(nodes) = assets.nodes.as_ref() else {
        return;
    };
    let dt = time.delta_secs();
    let game_over = *state.get() == GameState::GameOver;
    let move_threshold = visuals.player.move_speed_threshold.max(0.0);

    for (rig, mut player, mut director) in &mut players {
        let Ok((health, stamina, velocity, aim)) = survivors.get(rig.survivor) else {
            continue;
        };

        let alive = health.current > 0.0 && !game_over;
        if !alive {
            apply_death(&mut player, &mut director, nodes);
            continue;
        }
        if director.dead {
            director.dead = false;
            player.stop_all();
            prime_looped(&mut player, nodes);
        }

        let horizontal = Vec3::new(velocity.linear.x, 0.0, velocity.linear.z);
        let speed = horizontal.length();
        let moving = speed > move_threshold;

        let aim_dir = if aim.0.length_squared() > 1e-4 {
            aim.0.normalize()
        } else {
            Vec3::NEG_Z
        };
        let facing = if stamina.is_sprinting && moving {
            horizontal.normalize()
        } else {
            aim_dir
        };
        let right = facing.cross(Vec3::Y);
        let local = Vec2::new(horizontal.dot(right), horizontal.dot(facing));

        let reloading = inventories
            .get(rig.survivor)
            .ok()
            .and_then(WeaponInventory::reload_progress)
            .is_some();
        let aiming = stances
            .get(rig.survivor)
            .map(|stance| stance.aiming)
            .unwrap_or(false);
        // Missing assets degrade to relaxed arms instead of freezing the upper body.
        let reloading = reloading && nodes.get("reload").is_some();
        let aiming = aiming && nodes.get("aim").is_some();

        let targets = stance::targets(aiming, reloading);
        let action_blend = catalog.action_blend_secs;
        director.aim = blend::approach(director.aim, targets.aim, dt, action_blend);
        director.reload = blend::approach(director.reload, targets.reload, dt, action_blend);
        let relaxed = 1.0 - director.aim.max(director.reload).clamp(0.0, 1.0);

        let targets = locomotion_targets(local, nodes, stamina.is_sprinting, moving);
        let loco_blend = catalog.locomotion_blend_secs;
        director.idle = blend::approach(director.idle, targets.idle, dt, loco_blend);
        director.sprint = blend::approach(director.sprint, targets.sprint, dt, loco_blend);
        director.locomotion.forward = blend::approach(
            director.locomotion.forward,
            targets.weights.forward,
            dt,
            loco_blend,
        );
        director.locomotion.back = blend::approach(
            director.locomotion.back,
            targets.weights.back,
            dt,
            loco_blend,
        );
        director.locomotion.left = blend::approach(
            director.locomotion.left,
            targets.weights.left,
            dt,
            loco_blend,
        );
        director.locomotion.right = blend::approach(
            director.locomotion.right,
            targets.weights.right,
            dt,
            loco_blend,
        );

        let playback = if stamina.is_sprinting && moving {
            catalog.sprint_playback_speed.max(0.01)
        } else {
            blend::speed_scale(
                speed,
                catalog.walk_speed_ref,
                catalog.speed_scale_min,
                catalog.speed_scale_max,
            )
        };

        for role in LOCOMOTION_ROLES {
            let Some(clip_nodes) = nodes.get(role) else {
                continue;
            };
            let weight = match role {
                "idle" => director.idle,
                "walk_fwd" => director.locomotion.forward,
                "walk_back" => director.locomotion.back,
                "sprint" => director.sprint,
                "strafe_l" => director.locomotion.left,
                "strafe_r" => director.locomotion.right,
                _ => 0.0,
            };
            let base_speed = clip_nodes.speed.max(0.01);
            let clip_speed = if role == "idle" {
                base_speed
            } else {
                playback * base_speed
            };
            play_looped(&mut player, clip_nodes.lower_node(), weight, clip_speed);
            play_looped(
                &mut player,
                clip_nodes.upper_node(),
                weight * relaxed,
                clip_speed,
            );
        }

        let aim_weight = director.aim * (1.0 - director.reload);
        if let Some(clip_nodes) = nodes.get("aim") {
            play_action(&mut player, clip_nodes, aim_weight);
        }
        if let Some(clip_nodes) = nodes.get("reload") {
            let was_reloading = director.reload_active;
            if reloading
                && !was_reloading
                && let Some(node) = clip_nodes.action_node()
            {
                player.play(node).replay();
            }
            director.reload_active = reloading;
            play_action(&mut player, clip_nodes, director.reload);

            if reloading
                && let Some(progress) = inventories
                    .get(rig.survivor)
                    .ok()
                    .and_then(WeaponInventory::reload_progress)
                && let Some(node) = clip_nodes.action_node()
            {
                let duration = clips
                    .get(&clip_nodes.clip)
                    .map(AnimationClip::duration)
                    .unwrap_or(0.0);
                if duration > 0.0
                    && let Some(active) = player.animation_mut(node)
                {
                    active.set_seek_time(progress * duration);
                }
            }
        }
    }
}

struct Locomotion {
    idle: f32,
    sprint: f32,
    weights: Weights,
}

fn locomotion_targets(
    local: Vec2,
    nodes: &GraphNodes,
    sprinting: bool,
    moving: bool,
) -> Locomotion {
    if !moving {
        return Locomotion {
            idle: if nodes.get("idle").is_some() {
                1.0
            } else {
                0.0
            },
            sprint: 0.0,
            weights: Weights::default(),
        };
    }
    if sprinting && nodes.get("sprint").is_some() {
        return Locomotion {
            idle: 0.0,
            sprint: 1.0,
            weights: Weights::default(),
        };
    }

    let mut weights = Weights::from_local(local);
    if nodes.get("walk_back").is_none() {
        weights = weights.fallback_back_to_forward();
    }
    if nodes.get("strafe_l").is_none() || nodes.get("strafe_r").is_none() {
        weights = weights.fallback_sides_to_forward();
    }
    if weights.is_idle() {
        weights.forward = 1.0;
    }
    Locomotion {
        idle: 0.0,
        sprint: 0.0,
        weights,
    }
}

fn apply_death(player: &mut AnimationPlayer, director: &mut DirectorState, nodes: &GraphNodes) {
    if director.dead {
        return;
    }
    player.stop_all();
    if let Some(death) = nodes.death {
        let active = player.play(death);
        active.set_repeat(RepeatAnimation::Never);
        active.set_speed(1.0);
        active.set_weight(1.0);
    }
    director.dead = true;
}

fn play_looped(
    player: &mut AnimationPlayer,
    node: Option<AnimationNodeIndex>,
    weight: f32,
    speed: f32,
) {
    let Some(node) = node else {
        return;
    };
    let active = player.play(node);
    active.repeat();
    active.set_speed(speed.max(0.01));
    active.set_weight(weight.clamp(0.0, 1.0));
}

fn play_action(player: &mut AnimationPlayer, clip: &ClipNodes, weight: f32) {
    let Some(node) = clip.action_node() else {
        return;
    };
    let active = player.play(node);
    if clip.looped {
        active.repeat();
    } else {
        active.set_repeat(RepeatAnimation::Never);
    }
    active.set_speed(clip.speed.max(0.01));
    active.set_weight(weight.clamp(0.0, 1.0));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stationary_selection_is_idle_weighted() {
        let nodes = GraphNodes::default();
        let result = locomotion_targets(Vec2::ZERO, &nodes, false, false);
        assert_eq!(result.idle, 0.0, "no idle clip in an empty graph");
        assert!(result.weights.is_idle());
    }

    #[test]
    fn missing_cardinals_fall_back_to_forward() {
        let nodes = GraphNodes::default();
        let result = locomotion_targets(Vec2::new(1.0, -1.0), &nodes, false, true);
        assert!((result.weights.forward - 1.0).abs() < 0.001);
        assert!(result.weights.left < 0.001);
        assert!(result.weights.right < 0.001);
    }

    #[test]
    fn sprint_replaces_directional_blend() {
        let nodes = GraphNodes::default();
        let result = locomotion_targets(Vec2::new(0.0, 1.0), &nodes, true, true);
        assert_eq!(result.sprint, 0.0, "no sprint clip in an empty graph");
        assert!((result.weights.forward - 1.0).abs() < 0.001);
    }
}
