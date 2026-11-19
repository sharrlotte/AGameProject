use bevy::prelude::*;

use crate::content::team::{DEFAULT_TEAM_ID, Team};

use super::def::{MissionDef, ShipSpawn};
use crate::config::{GameConfigValue, GameSetting};
use crate::content::block::BlockRegistry;
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

pub fn formation(anchor: Vec2, index: usize, count: usize, spacing: f32) -> Vec2 {
    let offset = (index as f32 - (count.saturating_sub(1) as f32) / 2.0) * spacing;
    anchor + Vec2::Y * offset
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
    for (index, design) in fleet.iter().enumerate() {
        let team = Team::new(DEFAULT_TEAM_ID);
        let skill = resolve_skill(None, team, None, setting.ai_level, &ai_cfg.0);
        spawn_ship_entity(
            commands,
            registry,
            config,
            design,
            team,
            formation(mission.spawn, index, fleet.len(), spacing),
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
    fn three_ships_are_centered_on_anchor() {
        let anchor = Vec2::new(5000.0, 0.0);
        assert_eq!(formation(anchor, 0, 3, 400.0), anchor - Vec2::Y * 400.0);
        assert_eq!(formation(anchor, 1, 3, 400.0), anchor);
        assert_eq!(formation(anchor, 2, 3, 400.0), anchor + Vec2::Y * 400.0);
    }

    #[test]
    fn two_ships_straddle_anchor() {
        assert_eq!(formation(Vec2::ZERO, 0, 2, 400.0), Vec2::Y * -200.0);
        assert_eq!(formation(Vec2::ZERO, 1, 2, 400.0), Vec2::Y * 200.0);
    }

    #[test]
    fn single_ship_sits_at_anchor() {
        assert_eq!(formation(Vec2::X, 0, 1, 400.0), Vec2::X);
    }
}
