use bevy::animation::RepeatAnimation;
use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;
use data::GameState;

use crate::anim::blend::{self, AnchorWeights, MAX_ANCHORS, approach, wrap_turn};
use crate::anim::catalog::AnimCatalog;
use crate::anim::facing::PlayerFacing;
use crate::anim::graph::GraphNodes;
use crate::anim::{PlayerAnimAssets, PlayerRig};
use world::{Health, Stamina, Survivor};

/// One bound locomotion anchor with its clip nodes and normalized duration.
#[derive(Debug, Clone)]
pub struct AnchorClip {
    pub role: String,
    pub angle: f32,
    pub node: AnimationNodeIndex,
    pub duration: f32,
    pub speed: f32,
    pub sprint_node: Option<AnimationNodeIndex>,
    pub sprint_duration: f32,
    pub sprint_speed: f32,
}

/// The always-playing idle clip, driven separately from the directional anchors.
#[derive(Debug, Clone, Copy)]
pub struct IdleClip {
    pub node: AnimationNodeIndex,
    pub speed: f32,
}

/// Locomotion clips resolved once at bind time.
#[derive(Debug, Clone, Default)]
pub struct LocomotionClips {
    pub anchors: Vec<AnchorClip>,
    pub angles: Vec<f32>,
    pub idle: Option<IdleClip>,
}

/// Per-player animation state that must survive across frames.
#[derive(Component, Debug, Clone)]
pub struct DirectorState {
    pub idle: f32,
    pub weights: AnchorWeights,
    pub phase: f32,
    pub dominant: Option<usize>,
    pub dead: bool,
    pub sprint_blend: f32,
}

impl Default for DirectorState {
    fn default() -> Self {
        Self {
            idle: 1.0,
            weights: AnchorWeights::default(),
            phase: 0.0,
            dominant: None,
            dead: false,
            sprint_blend: 0.0,
        }
    }
}

/// Plays every locomotion clip at zero weight once, so the director can move
/// weights without restarting clips mid-gameplay.
pub fn prime_looped(player: &mut AnimationPlayer, locomotion: &LocomotionClips) {
    if let Some(idle) = locomotion.idle {
        let active = player.play(idle.node);
        active.repeat();
        active.set_weight(0.0);
        active.set_speed(idle.speed.max(0.01));
    }
    for anchor in &locomotion.anchors {
        let active = player.play(anchor.node);
        active.repeat();
        active.set_weight(0.0);
        active.set_speed(anchor.speed.max(0.01));
        if let Some(sprint_node) = anchor.sprint_node {
            let active = player.play(sprint_node);
            active.repeat();
            active.set_weight(0.0);
            active.set_speed(anchor.sprint_speed.max(0.01));
        }
    }
}

/// Resolves anchor nodes and clip durations, dropping clips that are not in the glTF.
pub fn resolve(
    catalog: &AnimCatalog,
    nodes: &GraphNodes,
    clips: &Assets<AnimationClip>,
) -> LocomotionClips {
    let loco = &catalog.locomotion;
    let idle = nodes.get(&loco.idle_role).and_then(|clip| {
        Some(IdleClip {
            node: clip.graph_node()?,
            speed: clip.speed,
        })
    });

    let mut anchors = Vec::with_capacity(loco.anchors.len().min(MAX_ANCHORS));
    let mut missing = Vec::new();
    for def in &loco.anchors {
        if anchors.len() >= MAX_ANCHORS {
            warn!("Locomotion defines more than {MAX_ANCHORS} anchors; extra anchors are ignored");
            break;
        }
        let Some(clip) = nodes.get(&def.role) else {
            missing.push(def.role.as_str());
            continue;
        };
        let Some(node) = clip.graph_node() else {
            missing.push(def.role.as_str());
            continue;
        };
        let duration = clips
            .get(&clip.clip)
            .map(AnimationClip::duration)
            .filter(|duration| duration.is_finite() && *duration > 0.0)
            .unwrap_or(0.0);

        // Resolve optional sprint variant clip.
        let (sprint_node, sprint_duration, sprint_speed) = match &def.sprint_role {
            Some(role) => match nodes.get(role) {
                Some(sclip) => match sclip.graph_node() {
                    Some(snode) => {
                        let sdur = clips
                            .get(&sclip.clip)
                            .map(AnimationClip::duration)
                            .filter(|d| d.is_finite() && *d > 0.0)
                            .unwrap_or(0.0);
                        (Some(snode), sdur, sclip.speed)
                    }
                    None => {
                        warn!("Sprint clip '{role}' has no graph node; dropping sprint variant");
                        (None, 0.0, 1.0)
                    }
                },
                None => {
                    warn!("Sprint clip '{role}' missing at bind; dropping sprint variant");
                    (None, 0.0, 1.0)
                }
            },
            None => (None, 0.0, 1.0),
        };

        anchors.push(AnchorClip {
            role: def.role.clone(),
            angle: wrap_turn(def.angle_rad()),
            node,
            duration,
            speed: clip.speed,
            sprint_node,
            sprint_duration,
            sprint_speed,
        });
    }
    if !missing.is_empty() {
        warn!("Locomotion anchors missing their clips: {missing:?}");
    }
    anchors.sort_by(|a, b| a.angle.total_cmp(&b.angle));
    let angles = anchors.iter().map(|anchor| anchor.angle).collect();

    LocomotionClips {
        anchors,
        angles,
        idle,
    }
}

/// Drives anchor weights, shared phase, playback speed, and death.
pub fn drive_player_animation(
    time: Res<Time>,
    state: Res<State<GameState>>,
    catalog: Res<AnimCatalog>,
    assets: Res<PlayerAnimAssets>,
    survivors: Query<(&Health, &Stamina, &Velocity), With<Survivor>>,
    facings: Query<&PlayerFacing>,
    mut players: Query<(&PlayerRig, &mut AnimationPlayer, &mut DirectorState)>,
) {
    let Some(locomotion) = assets.locomotion.as_ref() else {
        return;
    };
    let dt = time.delta_secs();
    let game_over = *state.get() == GameState::GameOver;
    let loco_cfg = &catalog.locomotion;
    let move_threshold = loco_cfg.move_speed_threshold.max(0.0);

    for (rig, mut player, mut director) in &mut players {
        let Ok((health, stamina, velocity)) = survivors.get(rig.survivor) else {
            continue;
        };

        let alive = alive(health, game_over);
        if !alive {
            apply_death(&mut player, &mut director, assets.as_ref());
            continue;
        }
        if director.dead {
            player.stop_all();
            prime_looped(&mut player, locomotion);
            *director = DirectorState::default();
        }

        let horizontal = Vec3::new(velocity.linear.x, 0.0, velocity.linear.z);
        let speed = horizontal.length();
        let moving = speed > move_threshold && !locomotion.anchors.is_empty();

        let yaw = facings
            .get(rig.root)
            .map(|facing| facing.yaw)
            .unwrap_or(0.0);
        let rotation = Quat::from_rotation_y(yaw);
        let local_angle = if moving {
            let forward = horizontal.dot(rotation * Vec3::NEG_Z);
            let right = horizontal.dot(rotation * Vec3::X);
            right.atan2(forward)
        } else {
            0.0
        };

        let targets = target_weights(local_angle, moving, locomotion.angles.as_slice());
        director.idle = approach(director.idle, targets.idle, dt, loco_cfg.blend_secs);
        resize_weights(&mut director.weights, locomotion.anchors.len());
        for index in 0..director.weights.len() {
            let current = director.weights.get(index);
            director.weights.set(
                index,
                approach(current, targets.weights.get(index), dt, loco_cfg.blend_secs),
            );
        }
        let idle = director.idle;
        director.weights.normalize(idle);

        let sprint_target = if stamina.is_sprinting && moving { 1.0 } else { 0.0 };
        director.sprint_blend = approach(
            director.sprint_blend,
            sprint_target,
            dt,
            loco_cfg.gait_blend_secs,
        );

        let dominant = director.weights.dominant();
        if let Some(index) = dominant
            && director.dominant != Some(index)
            && let Some(anchor) = locomotion.anchors.get(index)
        {
            let seek = seek_secs(director.phase, anchor.duration);
            seek_player(&mut player, anchor.node, seek);
            if let Some(sprint_node) = anchor.sprint_node {
                let sprint_seek = seek_secs(director.phase, anchor.sprint_duration);
                seek_player(&mut player, sprint_node, sprint_seek);
            }
            director.dominant = Some(index);
        }

        // Advance phase with blended playback/duration for the dominant anchor.
        if moving
            && let Some(index) = dominant
            && let Some(anchor) = locomotion.anchors.get(index)
        {
            if anchor.sprint_node.is_some() {
                let blend = director.sprint_blend;
                let dur = anchor.duration + (anchor.sprint_duration - anchor.duration) * blend;
                let reference = loco_cfg.walk_speed_ref
                    + (loco_cfg.sprint_speed_ref - loco_cfg.walk_speed_ref) * blend;
                let pb = blend::speed_scale(
                    speed,
                    reference.max(0.01),
                    loco_cfg.speed_scale_min,
                    loco_cfg.speed_scale_max,
                );
                director.phase = advance_phase(director.phase, dt, pb, dur);
            } else {
                let walk_pb = blend::speed_scale(
                    speed,
                    loco_cfg.walk_speed_ref,
                    loco_cfg.speed_scale_min,
                    loco_cfg.speed_scale_max,
                );
                let sprint_pb = loco_cfg.sprint_playback_speed.max(0.01);
                let blended = walk_pb + (sprint_pb - walk_pb) * director.sprint_blend;
                director.phase = advance_phase(director.phase, dt, blended, anchor.duration);
            }
        }

        for (index, anchor) in locomotion.anchors.iter().enumerate() {
            let w = director.weights.get(index);
            if let Some(sprint_node) = anchor.sprint_node {
                let walk_weight = w * (1.0 - director.sprint_blend);
                let sprint_weight = w * director.sprint_blend;
                let walk_speed = anchor.speed.max(0.01)
                    * blend::speed_scale(
                        speed,
                        loco_cfg.walk_speed_ref,
                        loco_cfg.speed_scale_min,
                        loco_cfg.speed_scale_max,
                    );
                let sprint_speed = anchor.sprint_speed.max(0.01)
                    * blend::speed_scale(
                        speed,
                        loco_cfg.sprint_speed_ref,
                        loco_cfg.speed_scale_min,
                        loco_cfg.speed_scale_max,
                    );
                play_looped(&mut player, anchor.node, walk_weight, walk_speed);
                play_looped(&mut player, sprint_node, sprint_weight, sprint_speed);
            } else {
                let walk_pb = blend::speed_scale(
                    speed,
                    loco_cfg.walk_speed_ref,
                    loco_cfg.speed_scale_min,
                    loco_cfg.speed_scale_max,
                );
                let sprint_pb = loco_cfg.sprint_playback_speed.max(0.01);
                let blended = walk_pb + (sprint_pb - walk_pb) * director.sprint_blend;
                let clip_speed = anchor.speed.max(0.01) * blended;
                play_looped(&mut player, anchor.node, w, clip_speed);
            }
        }
        if let Some(idle) = locomotion.idle {
            play_looped(&mut player, idle.node, director.idle, idle.speed);
        }
    }
}

struct Targets {
    weights: AnchorWeights,
    idle: f32,
}

fn alive(health: &Health, game_over: bool) -> bool {
    health.current > 0.0 && !game_over
}

fn target_weights(angle: f32, moving: bool, anchors: &[f32]) -> Targets {
    if !moving || anchors.is_empty() {
        return Targets {
            weights: AnchorWeights::new(anchors.len()),
            idle: 1.0,
        };
    }
    Targets {
        weights: AnchorWeights::bracket(anchors, angle),
        idle: 0.0,
    }
}

fn resize_weights(weights: &mut AnchorWeights, count: usize) {
    if weights.len() != count.min(MAX_ANCHORS) {
        let old = *weights;
        let mut replacement = AnchorWeights::new(count);
        for index in 0..replacement.len() {
            replacement.set(index, old.get(index));
        }
        *weights = replacement;
    }
}

pub fn seek_secs(phase: f32, duration: f32) -> f32 {
    if !duration.is_finite() || duration <= 0.0 {
        return 0.0;
    }
    phase.clamp(0.0, 1.0) * duration
}

pub fn advance_phase(phase: f32, dt: f32, playback: f32, duration: f32) -> f32 {
    if !duration.is_finite() || duration <= 0.0 || !dt.is_finite() || dt <= 0.0 {
        return phase;
    }
    let next = phase + dt * playback.max(0.0) / duration;
    next.rem_euclid(1.0)
}

fn seek_player(player: &mut AnimationPlayer, node: AnimationNodeIndex, seek: f32) {
    if seek <= 0.0 {
        return;
    }
    if let Some(active) = player.animation_mut(node) {
        active.set_seek_time(seek);
    }
}

fn play_looped(player: &mut AnimationPlayer, node: AnimationNodeIndex, weight: f32, speed: f32) {
    let active = player.play(node);
    active.repeat();
    active.set_speed(speed.max(0.01));
    active.set_weight(weight.clamp(0.0, 1.0));
}

fn apply_death(
    player: &mut AnimationPlayer,
    director: &mut DirectorState,
    assets: &PlayerAnimAssets,
) {
    if director.dead {
        return;
    }
    player.stop_all();
    if let Some(nodes) = assets.nodes.as_ref()
        && let Some(death) = nodes.death
    {
        let active = player.play(death);
        active.set_repeat(RepeatAnimation::Never);
        active.set_speed(1.0);
        active.set_weight(1.0);
    }
    director.dead = true;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anim::graph::ClipNodes;
    use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

    fn dummy_clip() -> ClipNodes {
        ClipNodes {
            clip: Handle::default(),
            node: None,
            looped: true,
            speed: 1.0,
        }
    }

    #[test]
    fn stationary_targets_are_idle_weighted() {
        let targets = target_weights(0.4, false, &[0.0, FRAC_PI_2]);
        assert_eq!(targets.idle, 1.0);
        assert_eq!(targets.weights.sum(), 0.0);
    }

    #[test]
    fn moving_targets_split_on_diagonals() {
        let targets = target_weights(FRAC_PI_4, true, &[0.0, FRAC_PI_2, std::f32::consts::PI]);
        assert_eq!(targets.idle, 0.0);
        assert!((targets.weights.get(0) - 0.5).abs() < 1e-4);
        assert!((targets.weights.get(1) - 0.5).abs() < 1e-4);
        assert!((targets.weights.sum() - 1.0).abs() < 1e-4);
    }

    #[test]
    fn missing_anchors_fall_back_to_idle() {
        let targets = target_weights(0.0, true, &[]);
        assert_eq!(targets.idle, 1.0);
        assert!(targets.weights.is_empty());
    }

    #[test]
    fn resolve_drops_anchors_without_clips() {
        let mut nodes = GraphNodes::default();
        nodes.clips.insert("walk_fwd".to_string(), dummy_clip());
        let clips = Assets::<AnimationClip>::default();
        let locomotion = resolve(&AnimCatalog::default(), &nodes, &clips);
        assert!(locomotion.anchors.is_empty());
        assert!(locomotion.idle.is_none());
    }

    #[test]
    fn resolve_sorts_surviving_anchors() {
        let mut nodes = GraphNodes::default();
        let mut clip = dummy_clip();
        clip.node = Some(AnimationNodeIndex::new(0));
        for role in ["walk_back", "walk_fwd", "strafe_l", "strafe_r"] {
            nodes.clips.insert(role.to_string(), clip.clone());
        }
        let clips = Assets::<AnimationClip>::default();
        let locomotion = resolve(&AnimCatalog::default(), &nodes, &clips);
        assert_eq!(locomotion.anchors.len(), 4);
        assert!(locomotion.angles.windows(2).all(|pair| pair[0] <= pair[1]));
    }

    #[test]
    fn phase_seek_maps_normalized_time_into_duration() {
        assert_eq!(seek_secs(0.0, 2.0), 0.0);
        assert_eq!(seek_secs(0.5, 2.0), 1.0);
        assert_eq!(seek_secs(1.0, 2.0), 2.0);
        assert_eq!(seek_secs(0.5, 0.0), 0.0);
    }

    #[test]
    fn phase_advances_and_wraps() {
        let half = advance_phase(0.0, 0.5, 1.0, 2.0);
        assert!((half - 0.25).abs() < 1e-4);
        let wrapped = advance_phase(0.9, 0.5, 1.0, 2.0);
        assert!((wrapped - 0.15).abs() < 1e-4);
        assert_eq!(advance_phase(0.3, 1.0, 1.0, 0.0), 0.3);
    }

    #[test]
    fn death_and_game_over_suppress_animation() {
        let mut health = Health::new(100.0);
        assert!(alive(&health, false));
        assert!(!alive(&health, true));
        health.current = 0.0;
        assert!(!alive(&health, false));
    }
}
