use std::collections::HashMap;

use bevy::prelude::*;
use bevy_rapier2d::prelude::Velocity;

use super::component::{
    Capability, LastSeen, Sensor, SharedContact, Skill, TacticalBoard, TeamAlert, TeamIntel,
};
use super::config::{AiConfigValue, IntelBounds};
use crate::content::block::BlockOwner;
use crate::content::bullet::{Bullet, Laser, Missile};
use crate::content::damage::DamageEvent;
use crate::content::ship::{Ship, ShipStats};
use crate::content::team::Team;

#[derive(Clone, Copy)]
struct ShipSnap {
    entity: Entity,
    team: u8,
    position: Vec2,
    velocity: Vec2,
    heading: Vec2,
    threat: f32,
    has_fixed_gun: bool,
}

pub fn update_sensors_and_intel(
    time: Res<Time>,
    ai_cfg: Res<AiConfigValue>,
    mut board: ResMut<TacticalBoard>,
    mut damage_events: MessageReader<DamageEvent>,
    blocks: Query<&BlockOwner>,
    bullets: Query<&Bullet>,
    missiles: Query<&Missile>,
    lasers: Query<&Laser>,
    global_transforms: Query<&GlobalTransform>,
    mut ships: Query<(
        Entity,
        &GlobalTransform,
        &Team,
        &Ship,
        &ShipStats,
        &Capability,
        Option<&Velocity>,
        Option<&Skill>,
        Option<&mut Sensor>,
    )>,
) {
    let dt = time.delta_secs();
    let mut snapshots: Vec<ShipSnap> = Vec::new();
    let mut team_max_skill: HashMap<u8, u8> = HashMap::new();
    let mut ship_map: HashMap<Entity, (u8, Vec2)> = HashMap::new();

    for (entity, gt, team, ship, stats, cap, vel, skill, _) in &ships {
        let pos = gt.transform_point(ship.core_position.extend(0.0)).truncate();
        let velocity = vel.map_or(Vec2::ZERO, |v| v.linear);
        let heading = gt.up().truncate().normalize_or_zero();
        let threat = cap.offensive_dps.max(stats.max_health * 0.02);
        snapshots.push(ShipSnap {
            entity,
            team: team.id,
            position: pos,
            velocity,
            heading,
            threat,
            has_fixed_gun: cap.fixed_mount.is_some(),
        });
        ship_map.insert(entity, (team.id, pos));
        let s = skill.map_or(ai_cfg.0.default_enemy_level, |s| s.level());
        team_max_skill
            .entry(team.id)
            .and_modify(|cur| *cur = (*cur).max(s))
            .or_insert(s);
    }

    // Ensure every active team has a TeamPlan entry
    for &team_id in team_max_skill.keys() {
        board.teams.entry(team_id).or_default();
    }

    // Move previous live contacts into staging so unspotted living enemies become last_known
    let mut prev_contacts_by_team: HashMap<u8, HashMap<Entity, SharedContact>> = HashMap::new();
    for (&team_id, plan) in &mut board.teams {
        let prev = std::mem::take(&mut plan.intel.contacts);
        prev_contacts_by_team.insert(team_id, prev);
    }

    // Scan radar per ship and publish immediately into TeamIntel.contacts
    for (_entity, gt, team, ship, _, cap, _, _, sensor_opt) in &mut ships {
        let origin = gt.transform_point(ship.core_position.extend(0.0)).truncate();
        let range_sq = cap.max_radar_range * cap.max_radar_range;
        let plan = board.teams.entry(team.id).or_default();

        let mut local: Vec<(f32, Entity)> = Vec::new();
        for enemy in &snapshots {
            if enemy.team == team.id {
                continue;
            }
            let dist_sq = origin.distance_squared(enemy.position);
            if dist_sq <= range_sq {
                local.push((dist_sq, enemy.entity));
                plan.intel.contacts.insert(
                    enemy.entity,
                    SharedContact {
                        entity: enemy.entity,
                        position: enemy.position,
                        velocity: enemy.velocity,
                        estimated_threat: enemy.threat,
                        has_fixed_gun: enemy.has_fixed_gun,
                        heading: enemy.heading,
                    },
                );
                plan.intel.last_known.remove(&enemy.entity);
            }
        }
        if let Some(mut sensor) = sensor_opt {
            local.sort_by(|a, b| a.0.total_cmp(&b.0));
            local.truncate(ai_cfg.0.bounds.max_local_contacts);
            sensor.local_contacts = local.into_iter().map(|(_, e)| e).collect();
        }
    }

    // Decay memory & alerts; migrate lost living targets to last_known (bounded)
    for (&team_id, plan) in &mut board.teams {
        let skill_lvl = team_max_skill
            .get(&team_id)
            .copied()
            .unwrap_or(ai_cfg.0.default_enemy_level);
        let mem_secs = ai_cfg.0.curves.memory_secs.sample(skill_lvl);

        tick_team_memory(&mut plan.intel, dt, ai_cfg.0.bounds.alert_spotted_radius);

        if let Some(prev) = prev_contacts_by_team.remove(&team_id) {
            for (enemy_entity, contact) in prev {
                if !plan.intel.contacts.contains_key(&enemy_entity)
                    && ship_map.contains_key(&enemy_entity)
                {
                    plan.intel.last_known.insert(
                        enemy_entity,
                        LastSeen {
                            position: contact.position,
                            velocity: contact.velocity,
                            ttl: mem_secs,
                        },
                    );
                }
            }
        }
        trim_last_known(&mut plan.intel.last_known, ai_cfg.0.bounds.max_last_known);
    }

    // Record shared DamageEvent hit alerts for unseen attackers
    for event in damage_events.read() {
        let Ok(owner) = blocks.get(event.target) else {
            continue;
        };

        let Some(&(victim_team, victim_pos)) = ship_map.get(&owner.body) else {
            continue;
        };

        let Some(alert_pos) =
            resolve_attacker_position(event.attacker, victim_pos, &bullets, &missiles, &lasers, &global_transforms)
        else {
            continue;
        };

        let skill_lvl = team_max_skill
            .get(&victim_team)
            .copied()
            .unwrap_or(ai_cfg.0.default_enemy_level);

        let ttl = ai_cfg.0.curves.alert_ttl_secs.sample(skill_lvl);
        let plan = board.teams.entry(victim_team).or_default();

        let spotted_sq =
            ai_cfg.0.bounds.alert_spotted_radius * ai_cfg.0.bounds.alert_spotted_radius;
        let already_spotted = plan
            .intel
            .contacts
            .values()
            .any(|c| c.position.distance_squared(alert_pos) <= spotted_sq);

        if !already_spotted {
            push_or_refresh_alert(&mut plan.intel.alerts, alert_pos, ttl, &ai_cfg.0.bounds);
        }
    }
}

fn trim_last_known(last_known: &mut HashMap<Entity, LastSeen>, max_entries: usize) {
    while last_known.len() > max_entries {
        let Some(oldest) = last_known
            .iter()
            .min_by(|a, b| a.1.ttl.total_cmp(&b.1.ttl))
            .map(|(e, _)| *e)
        else {
            break;
        };
        last_known.remove(&oldest);
    }
}

fn tick_team_memory(intel: &mut TeamIntel, dt: f32, spotted_radius: f32) {
    let spotted_sq = spotted_radius * spotted_radius;
    intel.last_known.retain(|entity, seen| {
        if intel.contacts.contains_key(entity) {
            return false;
        }
        seen.ttl -= dt;
        seen.ttl > 0.0
    });

    intel.alerts.retain_mut(|alert| {
        alert.ttl -= dt;
        if alert.ttl <= 0.0 {
            return false;
        }
        let spotted_near = intel
            .contacts
            .values()
            .any(|c| c.position.distance_squared(alert.position) <= spotted_sq);
        !spotted_near
    });
}

fn push_or_refresh_alert(
    alerts: &mut Vec<TeamAlert>,
    position: Vec2,
    ttl: f32,
    bounds: &IntelBounds,
) {
    let merge_sq = bounds.alert_merge_radius * bounds.alert_merge_radius;
    if let Some(existing) = alerts
        .iter_mut()
        .find(|a| a.position.distance_squared(position) <= merge_sq)
    {
        existing.position = position;
        existing.ttl = ttl;
    } else if alerts.len() < bounds.max_team_alerts {
        alerts.push(TeamAlert { position, ttl });
    } else if let Some(oldest) = alerts.iter_mut().min_by(|a, b| a.ttl.total_cmp(&b.ttl)) {
        *oldest = TeamAlert { position, ttl };
    }
}

fn resolve_attacker_position(
    attacker: Entity,
    victim_pos: Vec2,
    bullets: &Query<&Bullet>,
    missiles: &Query<&Missile>,
    lasers: &Query<&Laser>,
    transforms: &Query<&GlobalTransform>,
) -> Option<Vec2> {
    if let Ok(bullet) = bullets.get(attacker) {
        return Some(bullet.shooted_at);
    }
    if let Ok(missile) = missiles.get(attacker) {
        return Some(missile.shooted_at);
    }
    if let Ok(Laser::Init { origin, .. }) = lasers.get(attacker) {
        return Some(*origin);
    }
    if let Ok(gt) = transforms.get(attacker) {
        let pos = gt.translation().truncate();
        if pos.distance_squared(victim_pos) > 1.0 {
            return Some(pos);
        }
    }
    None
}
