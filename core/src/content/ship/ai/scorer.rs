use std::collections::HashMap;

use bevy::prelude::*;

use super::component::{
    Brain, Capability, Posture, Readiness, Role, SharedContact, Skill, SupportKind, TacticalBoard,
    Tactic, TeamPlan,
};
use super::config::{AiConfig, AiConfigValue};
use super::readiness::update_booster_shedding;
use crate::content::block::BlockOwner;
use crate::content::block::feature::power::Booster;
use crate::content::ship::Ship;
use crate::content::team::Team;
use crate::resource::ControlledShip;

pub struct EvalContext<'a> {
    pub entity: Entity,
    pub position: Vec2,
    pub spawn_position: Vec2,
    pub skill: u8,
    pub cap: &'a Capability,
    pub readiness: &'a Readiness,
    pub plan: &'a TeamPlan,
    pub cfg: &'a AiConfig,
    pub current: &'a Tactic,
    pub nearest_contact: Option<SharedContact>,
    pub preferred_contact: Option<SharedContact>,
    pub vip_position: Option<Vec2>,
}

pub trait Scorer: Send + Sync {
    fn score(&self, ctx: &EvalContext) -> Option<(Tactic, f32)>;
}

pub struct RetreatScorer;
pub struct RechargeScorer;
pub struct SupportScorer;
pub struct EngageScorer;
pub struct RegroupScorer;
pub struct InvestigateScorer;
pub struct SearchScorer;

impl Scorer for RetreatScorer {
    fn score(&self, ctx: &EvalContext) -> Option<(Tactic, f32)> {
        if !ctx.cap.can_move {
            return None;
        }

        let threat = ctx.nearest_contact?;
        if ctx.readiness.disarmed {
            return Some((
                Tactic::Retreat {
                    threat: threat.entity,
                },
                ctx.cfg.weights.retreat_max * 0.94,
            ));
        }

        let threshold = ctx.cfg.weights.retreat_hp_threshold;
        if ctx.readiness.hp_ratio >= threshold {
            return None;
        }

        let urgency = ((threshold - ctx.readiness.hp_ratio) / threshold.max(0.01)).clamp(0.0, 1.0);
        let score = (0.72 + urgency * 0.28).min(ctx.cfg.weights.retreat_max);
        Some((
            Tactic::Retreat {
                threat: threat.entity,
            },
            score,
        ))
    }
}

impl Scorer for RechargeScorer {
    fn score(&self, ctx: &EvalContext) -> Option<(Tactic, f32)> {
        if ctx.skill < ctx.cfg.unlocks.power_management_level
            || !ctx.cap.can_move
            || !ctx.cap.uses_energy_weapons
        {
            return None;
        }

        let threat = ctx.nearest_contact?;
        let threshold = ctx.cfg.weights.recharge_energy_threshold;
        if ctx.readiness.energy_ratio >= threshold {
            return None;
        }

        let ct = &ctx.cfg.combat_tuning;
        let dist = ctx.position.distance(threat.position);
        if dist > ctx.cap.max_range * ct.recharge_threat_range_factor {
            return None;
        }

        let deficit = ((threshold - ctx.readiness.energy_ratio) / threshold.max(0.01)).clamp(0.0, 1.0);
        let safe_range =
            (ctx.cap.max_range * ct.recharge_safe_range_factor).max(ct.recharge_min_safe_range);

        Some((
            Tactic::Recharge {
                threat: threat.entity,
                safe_range,
            },
            (0.60 + deficit * 0.24).min(ctx.cfg.weights.recharge_max),
        ))
    }
}

impl Scorer for SupportScorer {
    fn score(&self, ctx: &EvalContext) -> Option<(Tactic, f32)> {
        let vip = ctx.plan.vip_ally?;
        if vip == ctx.entity {
            return None;
        }

        let wants_support = matches!(ctx.cap.role, Role::Support | Role::Escort)
            || (ctx.skill >= ctx.cfg.unlocks.support_coordination_level
                && ctx.cap.booster_count > 0
                && ctx.cap.role == Role::Support);
        if !wants_support {
            return None;
        }

        let kind = if ctx.cap.booster_count > 0 && ctx.cap.role == Role::Support {
            SupportKind::Overdrive
        } else {
            SupportKind::Screen
        };
        
        let bonus = if ctx
            .vip_position
            .is_some_and(|p| p.distance(ctx.position) > ctx.cfg.combat_tuning.support_approach_bonus_dist)
        {
            0.06
        } else {
            0.0
        };

        Some((
            Tactic::Support { ally: vip, kind },
            (ctx.cfg.weights.support_base + bonus).clamp(0.0, 1.0),
        ))
    }
}

impl Scorer for EngageScorer {
    fn score(&self, ctx: &EvalContext) -> Option<(Tactic, f32)> {
        if ctx.readiness.disarmed {
            return None;
        }

        let target = ctx.preferred_contact.or(ctx.nearest_contact)?;
        let (posture, range) = select_posture_and_range(ctx);
        let range_score = if ctx.position.distance(target.position) <= ctx.cap.max_range * 1.2 {
            0.12
        } else {
            0.04
        };

        Some((
            Tactic::Engage {
                target: target.entity,
                range,
                posture,
            },
            (ctx.cfg.weights.engage_base + range_score).clamp(0.0, 1.0),
        ))
    }
}

impl Scorer for RegroupScorer {
    fn score(&self, ctx: &EvalContext) -> Option<(Tactic, f32)> {
        if !ctx.cap.can_move || ctx.plan.ally_count <= 1 {
            return None;
        }
        let dist_to_rally = ctx.position.distance(ctx.plan.rally_point);
        if dist_to_rally <= ctx.cfg.weights.regroup_ally_distance {
            return None;
        }
        if let Some(enemy) = ctx.nearest_contact
            && ctx.position.distance(enemy.position) <= ctx.cap.optimal_range * 1.1
        {
            return None;
        }
        Some((
            Tactic::Regroup {
                position: ctx.plan.rally_point,
            },
            ctx.cfg.weights.regroup_base,
        ))
    }
}

impl Scorer for InvestigateScorer {
    fn score(&self, ctx: &EvalContext) -> Option<(Tactic, f32)> {
        if !ctx.plan.intel.contacts.is_empty() {
            return None;
        }
        let target_pos = ctx
            .plan
            .intel
            .last_known
            .values()
            .min_by(|a, b| {
                a.position
                    .distance_squared(ctx.position)
                    .total_cmp(&b.position.distance_squared(ctx.position))
            })
            .map(|seen| seen.position)
            .or_else(|| {
                ctx.plan
                    .intel
                    .alerts
                    .iter()
                    .max_by(|a, b| a.ttl.total_cmp(&b.ttl))
                    .map(|alert| alert.position)
            })?;

        Some((
            Tactic::Investigate {
                position: target_pos,
            },
            ctx.cfg.weights.investigate_base,
        ))
    }
}

impl Scorer for SearchScorer {
    fn score(&self, ctx: &EvalContext) -> Option<(Tactic, f32)> {
        let next_waypoint = match *ctx.current {
            Tactic::Search { waypoint }
                if ctx.position.distance(waypoint) > ctx.cfg.investigate_arrival_radius =>
            {
                waypoint
            }
            _ => {
                let jitter = Vec2::new(
                    rand::random::<f32>() - 0.5,
                    rand::random::<f32>() - 0.5,
                ) * 2.0
                    * ctx.cfg.search_patrol_radius;
                ctx.spawn_position + jitter
            }
        };
        
        Some((
            Tactic::Search {
                waypoint: next_waypoint,
            },
            ctx.cfg.weights.search_base,
        ))
    }
}

fn select_posture_and_range(ctx: &EvalContext) -> (Posture, f32) {
    let cap = ctx.cap;
    let ct = &ctx.cfg.combat_tuning;
    let precision = ctx.cfg.curves.range_precision.sample(ctx.skill);
    if let Some(fixed) = cap.fixed_mount
        && (cap.fixed_dps_ratio >= ctx.cfg.role_tuning.sniper_fixed_dps_ratio
            || cap.role == Role::Sniper)
    {
        let hold = (fixed.range * ct.spinal_range_factor).max(cap.optimal_range * precision);
        return (Posture::SpinalAim, hold.max(ct.spinal_min_range));
    }

    if matches!(cap.role, Role::Sniper | Role::Skirmisher)
        || (cap.has_pulling_beam && cap.offensive_dps <= 0.0)
        || (ctx.readiness.hp_ratio < ct.kite_hp_threshold
            && ctx.skill >= ctx.cfg.unlocks.focus_fire_level)
    {
        return (
            Posture::Kite,
            (cap.optimal_range * precision).max(ct.kite_min_range),
        );
    }

    if cap.has_rotating_turrets {
        let range = if cap.role == Role::Brawler && cap.has_laser_beam {
            cap.optimal_range * ct.orbit_beam_range_factor
        } else {
            cap.optimal_range * ct.orbit_range_factor
        };
        return (Posture::Orbit, range.max(ct.orbit_min_range));
    }

    (
        Posture::Brawl,
        (cap.optimal_range * ct.brawl_range_factor).max(ct.brawl_min_range),
    )
}

pub fn select_tactic(ctx: &EvalContext) -> Tactic {
    let scorers: [&dyn Scorer; 7] = [
        &RetreatScorer,
        &RechargeScorer,
        &SupportScorer,
        &EngageScorer,
        &RegroupScorer,
        &InvestigateScorer,
        &SearchScorer,
    ];

    let mut best: Option<(Tactic, f32)> = None;
    for scorer in scorers {
        let Some((candidate, mut score)) = scorer.score(ctx) else {
            continue;
        };
        if ctx.current.same_kind(&candidate) {
            score += ctx.cfg.hysteresis;
        }
        if best.is_none_or(|(_, best_score)| score > best_score) {
            best = Some((candidate, score));
        }
    }

    best.map_or(
        Tactic::Search {
            waypoint: ctx.spawn_position,
        },
        |(tactic, _)| tactic,
    )
}

pub fn evaluate_utility_and_power(
    mut commands: Commands,
    time: Res<Time>,
    ai_cfg: Res<AiConfigValue>,
    board: Res<TacticalBoard>,
    controlled: Res<ControlledShip>,
    boosters: Query<(Entity, &BlockOwner), With<Booster>>,
    transforms: Query<&GlobalTransform>,
    ship_cores: Query<&Ship>,
    mut ships: Query<(
        Entity,
        &mut Brain,
        &Skill,
        &Capability,
        &Readiness,
        &Team,
        &GlobalTransform,
        &Ship,
    )>,
    mut boosters_by_ship: Local<HashMap<Entity, Vec<Entity>>>,
) {
    for list in boosters_by_ship.values_mut() {
        list.clear();
    }
    for (booster_entity, owner) in &boosters {
        boosters_by_ship
            .entry(owner.body)
            .or_default()
            .push(booster_entity);
    }

    let dt = time.delta();
    let dt_secs = time.delta_secs();
    let empty_plan = super::component::TeamPlan::default();

    for (entity, mut brain, skill, cap, readiness, team, gt, ship) in &mut ships {
        if controlled.controlling == Some(entity) {
            continue;
        }

        brain.strafe_timer -= dt_secs;
        if brain.strafe_timer <= 0.0 {
            brain.strafe_sign = -brain.strafe_sign;
            let min_s = ai_cfg.0.combat_tuning.strafe_flip_min_secs;
            let span = (ai_cfg.0.combat_tuning.strafe_flip_max_secs - min_s).max(0.1);
            brain.strafe_timer = min_s + rand::random::<f32>() * span;
        }

        let desired_interval = 1.0 / ai_cfg.0.curves.think_rate_hz.sample(skill.level()).max(0.1);
        if (brain.think_timer.duration().as_secs_f32() - desired_interval).abs() > 0.01 {
            brain
                .think_timer
                .set_duration(std::time::Duration::from_secs_f32(desired_interval));
        }
        brain.think_timer.tick(dt);
        if !brain.think_timer.just_finished() {
            continue;
        }

        let level = skill.level();
        update_booster_shedding(
            &mut commands,
            entity,
            &mut brain,
            level,
            cap,
            readiness,
            &ai_cfg.0,
            &boosters_by_ship,
        );

        let position = gt.transform_point(ship.core_position.extend(0.0)).truncate();
        let plan = board.teams.get(&team.id).unwrap_or(&empty_plan);
        let nearest_contact = plan
            .intel
            .contacts
            .values()
            .copied()
            .min_by(|a, b| {
                a.position
                    .distance_squared(position)
                    .total_cmp(&b.position.distance_squared(position))
            });
        let preferred_contact = if level >= ai_cfg.0.unlocks.focus_fire_level {
            plan.focus_target
                .and_then(|id| plan.intel.contacts.get(&id).copied())
                .or(nearest_contact)
        } else {
            nearest_contact
        };
        let vip_position = plan.vip_ally.and_then(|vip| {
            let vip_gt = transforms.get(vip).ok()?;
            let vip_ship = ship_cores.get(vip).ok()?;
            Some(
                vip_gt
                    .transform_point(vip_ship.core_position.extend(0.0))
                    .truncate(),
            )
        });

        let ctx = EvalContext {
            entity,
            position,
            spawn_position: brain.spawn_position,
            skill: level,
            cap,
            readiness,
            plan,
            cfg: &ai_cfg.0,
            current: &brain.current,
            nearest_contact,
            preferred_contact,
            vip_position,
        };
        brain.current = select_tactic(&ctx);
    }
}
