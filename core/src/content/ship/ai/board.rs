use std::collections::HashMap;

use bevy::prelude::*;

use super::component::{Capability, Role, SharedContact, TacticalBoard};
use super::config::{AiConfigValue, BoardTuning};
use crate::content::ship::{Ship, ShipStats};
use crate::content::team::Team;

#[derive(Clone, Copy)]
struct AllyCandidate {
    entity: Entity,
    position: Vec2,
    offensive_dps: f32,
    is_support: bool,
}

pub fn update_tactical_board(
    ai_cfg: Res<AiConfigValue>,
    mut board: ResMut<TacticalBoard>,
    ships: Query<(Entity, &GlobalTransform, &Team, &Ship, &ShipStats, &Capability)>,
) {
    let mut allies_by_team: HashMap<u8, Vec<AllyCandidate>> = HashMap::new();
    let mut hp_by_entity: HashMap<Entity, f32> = HashMap::new();

    for (entity, gt, team, ship, stats, cap) in &ships {
        let pos = gt.transform_point(ship.core_position.extend(0.0)).truncate();
        let hp_ratio = if stats.max_health > 0.0 {
            (stats.health / stats.max_health).clamp(0.0, 1.0)
        } else {
            1.0
        };
        hp_by_entity.insert(entity, hp_ratio);
        allies_by_team.entry(team.id).or_default().push(AllyCandidate {
            entity,
            position: pos,
            offensive_dps: cap.offensive_dps,
            is_support: matches!(cap.role, Role::Support | Role::Escort),
        });
    }

    board.teams.retain(|team_id, _| allies_by_team.contains_key(team_id));

    for (team_id, allies) in allies_by_team {
        let plan = board.teams.entry(team_id).or_default();
        plan.ally_count = allies.len();

        let count = allies.len().max(1) as f32;
        plan.rally_point = allies.iter().map(|a| a.position).sum::<Vec2>() / count;

        plan.vip_ally = allies
            .iter()
            .filter(|a| !a.is_support)
            .max_by(|a, b| {
                a.offensive_dps
                    .total_cmp(&b.offensive_dps)
                    .then_with(|| a.entity.cmp(&b.entity))
            })
            .or_else(|| {
                allies.iter().max_by(|a, b| {
                    a.offensive_dps
                        .total_cmp(&b.offensive_dps)
                        .then_with(|| a.entity.cmp(&b.entity))
                })
            })
            .map(|a| a.entity);

        let max_threat = plan
            .intel
            .contacts
            .values()
            .map(|c| c.estimated_threat)
            .fold(1.0_f32, f32::max);

        plan.focus_target = plan
            .intel
            .contacts
            .values()
            .max_by(|a, b| {
                let score_a = focus_score(
                    a,
                    plan.rally_point,
                    max_threat,
                    &hp_by_entity,
                    &ai_cfg.0.board_tuning,
                );
                let score_b = focus_score(
                    b,
                    plan.rally_point,
                    max_threat,
                    &hp_by_entity,
                    &ai_cfg.0.board_tuning,
                );
                score_a
                    .total_cmp(&score_b)
                    .then_with(|| b.entity.cmp(&a.entity))
            })
            .map(|c| c.entity);
    }
}

fn focus_score(
    contact: &SharedContact,
    rally_point: Vec2,
    max_threat: f32,
    hp_by_entity: &HashMap<Entity, f32>,
    tuning: &BoardTuning,
) -> f32 {
    let hp_ratio = hp_by_entity.get(&contact.entity).copied().unwrap_or(1.0);
    let damage_urgency = 1.0 - hp_ratio;
    let threat_norm = (contact.estimated_threat / max_threat.max(1.0)).clamp(0.0, 1.0);
    let dist_penalty = (contact.position.distance(rally_point)
        / tuning.focus_distance_scale.max(1.0))
    .clamp(0.0, tuning.focus_distance_penalty_cap);
    damage_urgency * tuning.focus_damage_weight + threat_norm * tuning.focus_threat_weight
        - dist_penalty
}
