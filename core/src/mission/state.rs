use bevy::prelude::*;

use crate::content::team::{DEFAULT_TEAM_ID, Team};

use super::def::{MissionDef, ShipSpawn};
use crate::config::{GameConfigValue, GameSetting};
use crate::constant::GRID_SIZE;
use crate::content::block::BlockRegistry;
use crate::content::body::BlockGrid;
use crate::content::ship::ShipDesign;
use crate::content::ship::ai::{AiConfig, AiConfigValue, Brain, Skill};
use crate::save::design::ShipCatalog;

#[derive(States, Default, Clone, Copy, Eq, Hash, PartialEq, Debug)]
pub enum BattlePhase {
    #[default]
    Idle,
    Loading,
    Active,
    Ended,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum BattleOutcome {
    Victory,
    Defeat,
}

#[derive(Resource, Default)]
pub struct BattleResult {
    pub started_at: Option<f32>,
    pub outcome: Option<BattleOutcome>,
    pub duration_secs: f32,
    pub mission_name: String,
}

#[derive(Resource, Default)]
pub struct BattleRestart(pub bool);

pub fn find_schematic<'a>(schematics: &'a [ShipDesign], name: &str) -> Option<&'a ShipDesign> {
    schematics
        .iter()
        .find(|s| s.name == name)
        .or_else(|| schematics.first())
}

pub fn resolve_skill(
    spawn_level: Option<u8>,
    team: Team,
    mission_level: Option<u8>,
    setting_level: u8,
    ai_cfg: &AiConfig,
) -> Skill {
    let raw = spawn_level.unwrap_or_else(|| {
        if team.is_player() {
            ai_cfg.default_ally_level
        } else {
            mission_level.unwrap_or_else(|| {
                if (1..=10).contains(&setting_level) {
                    setting_level
                } else {
                    ai_cfg.default_enemy_level
                }
            })
        }
    });
    Skill::new(raw)
}

fn spawn_ship_entity(
    commands: &mut Commands,
    registry: &BlockRegistry,
    config: &GameConfigValue,
    design: &ShipDesign,
    team: Team,
    position: Vec2,
    skill: Skill,
    ai_cfg: &AiConfig,
) {
    let Some((ship_entity, _)) = design.spawn(
        commands,
        registry,
        team,
        Transform::from_translation(position.extend(0.0)),
        config,
    ) else {
        return;
    };

    let think_hz = ai_cfg.curves.think_rate_hz.sample(skill.level());
    commands
        .entity(ship_entity)
        .insert((skill, Brain::new(position, think_hz)));
}

#[allow(clippy::too_many_arguments)]
fn spawn_ship(
    commands: &mut Commands,
    registry: &BlockRegistry,
    config: &GameConfigValue,
    schematics: &[ShipDesign],
    spawn: &ShipSpawn,
    mission_level: Option<u8>,
    setting: &GameSetting,
    ai_cfg: &AiConfig,
) {
    let Some(schematic) = find_schematic(schematics, &spawn.schematic_name) else {
        warn!(
            "Mission references schematic {:?}, which is not available",
            spawn.schematic_name
        );
        return;
    };

    let team = Team::new(spawn.team);
    let skill = resolve_skill(spawn.ai_level, team, mission_level, setting.ai_level, ai_cfg);
    spawn_ship_entity(
        commands,
        registry,
        config,
        schematic,
        team,
        spawn.position,
        skill,
        ai_cfg,
    );
}

pub fn hull_extent_y(design: &ShipDesign, registry: &BlockRegistry) -> Vec2 {
    let mut grid = BlockGrid::new();
    grid.rebuild_from_layout(&design.blocks, registry);

    let Some(min) = grid.cells.keys().map(|cell| cell.y).min() else {
        return Vec2::ZERO;
    };
    let Some(max) = grid.cells.keys().map(|cell| cell.y).max() else {
        return Vec2::ZERO;
    };

    Vec2::new(min as f32 * GRID_SIZE, (max + 1) as f32 * GRID_SIZE)
}

pub fn line_positions(anchor: Vec2, extents: &[Vec2], gap: f32) -> Vec<Vec2> {
    let total = extents
        .iter()
        .map(|extent| extent.y - extent.x)
        .sum::<f32>()
        + gap * extents.len().saturating_sub(1) as f32;

    let mut cursor = -total * 0.5;
    extents
        .iter()
        .map(|extent| {
            let offset = cursor - extent.x;
            cursor += extent.y - extent.x + gap;
            anchor + Vec2::Y * offset
        })
        .collect()
}

pub fn formation_positions(
    anchor: Vec2,
    fleet: &[ShipDesign],
    registry: &BlockRegistry,
    gap: f32,
) -> Vec<Vec2> {
    let extents = fleet
        .iter()
        .map(|design| hull_extent_y(design, registry))
        .collect::<Vec<_>>();
    line_positions(anchor, &extents, gap)
}

#[allow(clippy::too_many_arguments)]
pub fn spawn_mission(
    commands: &mut Commands,
    registry: &BlockRegistry,
    config: &GameConfigValue,
    mission: &MissionDef,
    fleet: &[ShipDesign],
    catalog: &ShipCatalog,
    setting: &GameSetting,
    ai_cfg: &AiConfigValue,
    spacing: f32,
) {
    let positions = formation_positions(mission.spawn, fleet, registry, spacing);
    for (design, position) in fleet.iter().zip(positions) {
        let team = Team::new(DEFAULT_TEAM_ID);
        let skill = resolve_skill(None, team, None, setting.ai_level, &ai_cfg.0);
        spawn_ship_entity(
            commands,
            registry,
            config,
            design,
            team,
            position,
            skill,
            &ai_cfg.0,
        );
    }

    for enemy in &mission.enemies {
        spawn_ship(
            commands,
            registry,
            config,
            catalog.designs(),
            enemy,
            mission.ai_level,
            setting,
            &ai_cfg.0,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hulls_are_separated_by_the_configured_gap() {
        let anchor = Vec2::new(5000.0, 0.0);
        let positions = line_positions(
            anchor,
            &[Vec2::new(-512.0, 512.0), Vec2::new(-288.0, 288.0)],
            50.0,
        );
        assert_eq!(positions[0], anchor - Vec2::Y * 313.0);
        assert_eq!(positions[1], anchor + Vec2::Y * 537.0);
    }

    #[test]
    fn three_hulls_are_centered_on_anchor() {
        let extents = [Vec2::new(-50.0, 50.0); 3];
        let positions = line_positions(Vec2::ZERO, &extents, 100.0);
        assert_eq!(positions[0], Vec2::Y * -200.0);
        assert_eq!(positions[1], Vec2::ZERO);
        assert_eq!(positions[2], Vec2::Y * 200.0);
    }

    #[test]
    fn single_ship_sits_at_anchor() {
        assert_eq!(
            line_positions(Vec2::X, &[Vec2::new(-50.0, 50.0)], 400.0),
            vec![Vec2::X]
        );
    }
}
