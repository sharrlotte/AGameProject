use std::collections::HashSet;
use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

use super::component::{Capability, Doctrine, FixedMount, Readiness, Role};
use super::config::{AiConfig, AiConfigValue, RoleTuning};
use crate::content::block::feature::component::Feature;
use crate::content::block::feature::turret::{TurretComponent, TurretPower};
use crate::content::block::{
    BlockOwner, BlockRegistry, Health, ShootPattern, ShootingProjectile, TargetingStrategy,
};
use crate::content::body::BlockGrid;
use crate::content::damage::DamageEvent;
use crate::content::ship::{Ship, ShipChanged, ShipStats};

pub fn compute_capability(
    grid: &BlockGrid,
    ship: &Ship,
    stats: &ShipStats,
    doctrine: Doctrine,
    registry: &BlockRegistry,
    ai_cfg: &AiConfig,
    live_healths: Option<&Query<&Health>>,
) -> Capability {
    let mut cap = Capability {
        max_radar_range: ai_cfg.base_core_radar_range,
        ..default()
    };

    let mut weighted_range_sum = 0.0;
    let mut fixed_dps = 0.0;
    let mut raw_thrust = 0.0;

    for block in grid.blocks() {
        let Some(def) = registry.get_symbol(&block.placement.block_type) else {
            continue;
        };

        let block_hp = live_healths
            .and_then(|q| q.get(block.entity).ok())
            .map_or(def.health, |h| h.health.max(0.0));

        let rel = block.placement.position_pixel()
            + def.get_pixel_size(block.placement.rotation) * 0.5
            - ship.core_position;

        if rel.length_squared() > 1.0 {
            let quadrant = if rel.y.abs() >= rel.x.abs() {
                if rel.y >= 0.0 { 0 } else { 2 }
            } else if rel.x >= 0.0 {
                1
            } else {
                3
            };

            cap.quadrant_hp[quadrant] += block_hp;
        }

        for feature in &def.features {
            classify_feature(
                feature,
                block.placement.rotation.turns() as f32 * FRAC_PI_2,
                &ai_cfg.role_tuning,
                &mut cap,
                &mut weighted_range_sum,
                &mut fixed_dps,
                &mut raw_thrust,
            );
        }
    }

    cap.can_move = raw_thrust > 0.0;
    cap.thrust_to_mass = if stats.mass > 0.0 {
        raw_thrust / stats.mass
    } else {
        0.0
    };

    if cap.offensive_dps > 0.0 {
        cap.optimal_range = weighted_range_sum / cap.offensive_dps;
        cap.fixed_dps_ratio = (fixed_dps / cap.offensive_dps).clamp(0.0, 1.0);
    } else if cap.has_pulling_beam {
        cap.optimal_range = cap.max_range * ai_cfg.role_tuning.turret_optimal_range_factor;
    } else {
        cap.optimal_range = cap.max_radar_range * ai_cfg.role_tuning.turret_optimal_range_factor;
    }

    cap.role = infer_role(&cap, doctrine, ai_cfg);
    cap
}

fn classify_feature(
    feature: &Feature,
    mount_angle: f32,
    tuning: &RoleTuning,
    cap: &mut Capability,
    weighted_range_sum: &mut f32,
    fixed_dps: &mut f32,
    raw_thrust: &mut f32,
) {
    match feature {
        Feature::Turret(turret) => {
            cap.max_radar_range = cap.max_radar_range.max(turret.range);
            if matches!(turret.strategy, TargetingStrategy::EnemyBullet) {
                cap.has_point_defense = true;
                cap.support_weight += tuning.point_defense_support_weight;
                return;
            }

            if let Some(power) = turret.power {
                cap.uses_energy_weapons = true;
                if let TurretPower::Drain { rate } = power {
                    cap.drain_rate += rate;
                }
            }
            if turret.ammo.is_some() {
                cap.uses_ammo_weapons = true;
            }

            match &turret.shooting_pattern {
                ShootPattern::PullingBeam(_) => {
                    cap.has_pulling_beam = true;
                    cap.support_weight += tuning.pulling_beam_support_weight;
                    cap.max_range = cap.max_range.max(turret.range);
                    return;
                }
                ShootPattern::LaserBeam(_) => cap.has_laser_beam = true,
                _ => {}
            }

            let dps = estimate_turret_dps(turret);
            cap.offensive_dps += dps;
            cap.max_range = cap.max_range.max(turret.range);
            *weighted_range_sum += dps * turret.range * tuning.turret_optimal_range_factor;

            if turret.turn_speed == 0.0 {
                *fixed_dps += dps;
                let replace = cap
                    .fixed_mount
                    .is_none_or(|prev| dps > prev.dps);
                if replace {
                    cap.fixed_mount = Some(FixedMount {
                        mount_angle,
                        range: turret.range,
                        dps,
                        aim_tolerance: turret.aim_tolerance,
                        projectile_speed: turret.shooting_pattern.projectile_speed(),
                    });
                }
            } else {
                cap.has_rotating_turrets = true;
            }
        }
        Feature::Booster(booster) => {
            cap.booster_count += 1;
            cap.booster_radius = cap.booster_radius.max(booster.radius);
            cap.booster_rate += booster.rate;
            cap.support_weight += tuning.booster_support_weight * booster.boost.max(1.0);
        }
        Feature::Thruster(thruster) => {
            *raw_thrust += thruster.force().max(0.0);
        }
        _ => {}
    }
}

fn estimate_turret_dps(turret: &TurretComponent) -> f32 {
    let barrels = turret.barrels.len().max(1) as f32;
    
    match &turret.shooting_pattern {
        ShootPattern::Single(s) => {
            let dmg = projectile_damage(&s.projectile);
            dmg * barrels / s.fire_rate.max(0.1)
        }
        ShootPattern::Burst(b) => {
            let dmg = projectile_damage(&b.projectile);
            let cycle = (b.fire_rate + b.delay * b.amount.max(1) as f32).max(0.1);
            dmg * b.amount.max(1) as f32 / cycle
        }
        ShootPattern::LaserBeam(l) => {
            let cycle = (l.cooldown + l.duration).max(0.1);
            l.damage * 60.0 * l.duration / cycle
        }
        ShootPattern::PullingBeam(_) => 0.0,
    }
}

fn projectile_damage(projectile: &ShootingProjectile) -> f32 {
    match projectile {
        ShootingProjectile::Bullet { damage, .. } | ShootingProjectile::Laser { damage, .. } => {
            *damage
        }
        ShootingProjectile::Missile {
            damage,
            blast_damage,
            ..
        } => damage + blast_damage,
    }
}

pub fn infer_role(cap: &Capability, doctrine: Doctrine, ai_cfg: &AiConfig) -> Role {
    let tuning = &ai_cfg.role_tuning;
    match doctrine {
        Doctrine::Brawler => Role::Brawler,
        Doctrine::Skirmisher => Role::Skirmisher,
        Doctrine::Sniper => Role::Sniper,
        Doctrine::Support => Role::Support,
        Doctrine::Escort => Role::Escort,
        Doctrine::Auto => {
            if cap.support_weight > cap.offensive_dps * ai_cfg.support_auto_ratio {
                Role::Support
            } else if cap.fixed_dps_ratio >= tuning.sniper_fixed_dps_ratio
                && cap.optimal_range >= tuning.sniper_min_optimal_range
            {
                Role::Sniper
            } else if cap.optimal_range >= tuning.skirmisher_min_optimal_range
                && cap.thrust_to_mass >= tuning.skirmisher_min_thrust_to_mass
            {
                Role::Skirmisher
            } else {
                Role::Brawler
            }
        }
    }
}

pub fn rebuild_capabilities_on_change(
    mut events: MessageReader<ShipChanged>,
    mut ships: Query<(&Ship, &ShipStats, &BlockGrid, Option<&Doctrine>, &mut Capability)>,
    healths: Query<&Health>,
    registry: Res<BlockRegistry>,
    ai_cfg: Res<AiConfigValue>,
) {
    for event in events.read() {
        let Ok((ship, stats, grid, doctrine, mut cap)) = ships.get_mut(event.ship) else {
            continue;
        };
        *cap = compute_capability(
            grid,
            ship,
            stats,
            doctrine.copied().unwrap_or_default(),
            &registry,
            &ai_cfg.0,
            Some(&healths),
        );
    }
}

pub fn refresh_quadrant_hp_on_damage(
    mut events: MessageReader<DamageEvent>,
    blocks: Query<&BlockOwner>,
    mut ships: Query<(&Ship, &BlockGrid, &mut Capability)>,
    healths: Query<&Health>,
    registry: Res<BlockRegistry>,
) {
    let mut dirty = HashSet::new();
    for event in events.read() {
        if let Ok(owner) = blocks.get(event.target) {
            dirty.insert(owner.body);
        }
    }

    for body in dirty {
        let Ok((ship, grid, mut cap)) = ships.get_mut(body) else {
            continue;
        };
        let mut quadrant_hp = [0.0; 4];
        for block in grid.blocks() {
            let Some(def) = registry.get_symbol(&block.placement.block_type) else {
                continue;
            };
            let hp = healths
                .get(block.entity)
                .map_or(def.health, |h| h.health.max(0.0));
            let rel = block.placement.position_pixel()
                + def.get_pixel_size(block.placement.rotation) * 0.5
                - ship.core_position;
            if rel.length_squared() > 1.0 {
                let q = if rel.y.abs() >= rel.x.abs() {
                    if rel.y >= 0.0 { 0 } else { 2 }
                } else if rel.x >= 0.0 {
                    1
                } else {
                    3
                };
                quadrant_hp[q] += hp;
            }
        }
        cap.quadrant_hp = quadrant_hp;
    }
}

pub fn init_missing_capabilities(
    mut commands: Commands,
    ships: Query<
        (Entity, &Ship, &ShipStats, &BlockGrid, Option<&Doctrine>),
        Without<Capability>,
    >,
    healths: Query<&Health>,
    registry: Res<BlockRegistry>,
    ai_cfg: Res<AiConfigValue>,
) {
    for (entity, ship, stats, grid, doctrine) in &ships {
        let cap = compute_capability(
            grid,
            ship,
            stats,
            doctrine.copied().unwrap_or_default(),
            &registry,
            &ai_cfg.0,
            Some(&healths),
        );
        commands.entity(entity).insert((cap, Readiness::default()));
    }
}
