use bevy::prelude::*;

use crate::config::{ConfigState, GameConfig, GameSettings, ShipSpawn};
use crate::constant::GRID_SIZE;
use crate::content::block::BlockRegistry;
use crate::content::body::BlockGrid;
use crate::content::ship::ai::{AiConfig, Skill};
use crate::content::ship::spawn::SpawnEnv;
use crate::content::ship::ShipDesign;
use crate::content::team::{DEFAULT_TEAM_ID, Team};
use crate::resource::{DeployPayload, SceneState};
use crate::save::design::ShipCatalog;
use crate::save::world::WorldContext;
use crate::simulation::SimulationSet;
use crate::validate::SkillLevel;

#[derive(Resource, Default)]
struct SkirmishPending(bool);

pub struct SkirmishPlugin;

impl Plugin for SkirmishPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SkirmishPending>()
            .add_systems(OnEnter(SceneState::Game), begin_skirmish)
            .add_systems(
                Update,
                spawn_skirmish
                    .run_if(in_state(SceneState::Game))
                    .run_if(in_state(ConfigState::Ready))
                    .run_if(|pending: Res<SkirmishPending>| pending.0)
                    .in_set(SimulationSet::Present),
            );
    }
}

fn begin_skirmish(context: Res<WorldContext>, mut pending: ResMut<SkirmishPending>) {
    pending.0 = !context.is_restoring();
}

#[allow(clippy::too_many_arguments)]
fn spawn_skirmish(
    mut commands: Commands,
    registry: Res<BlockRegistry>,
    config: Res<GameConfig>,
    setting: Res<GameSettings>,
    fleet: Res<DeployPayload>,
    catalog: Res<ShipCatalog>,
    mut pending: ResMut<SkirmishPending>,
) {
    let def = &config.skirmish;

    let env = SpawnEnv::new(&registry, &config, &setting, &config.ai);
    spawn_fleet(&mut commands, &env, &fleet.fleet, def.spawn, config.sim.fleet_spacing.get());
    for enemy in &def.enemies {
        spawn_enemy(&mut commands, &env, catalog.designs(), enemy, def.ai_level);
    }

    pending.0 = false;
}

pub fn resolve_skill(
    spawn_level: Option<SkillLevel>,
    team: Team,
    mission_level: Option<SkillLevel>,
    setting_level: SkillLevel,
    ai_cfg: &AiConfig,
) -> Skill {
    let level = spawn_level.unwrap_or_else(|| {
        if team.is_player() {
            ai_cfg.default_ally_level
        } else {
            mission_level.unwrap_or(setting_level)
        }
    });
    Skill::new(level.get())
}

pub fn find_schematic<'a>(designs: &'a [ShipDesign], name: &str) -> Option<&'a ShipDesign> {
    designs
        .iter()
        .find(|design| design.name == name)
        .or_else(|| designs.first())
}

fn spawn_enemy(
    commands: &mut Commands,
    env: &SpawnEnv,
    designs: &[ShipDesign],
    spawn: &ShipSpawn,
    mission_level: Option<SkillLevel>,
) {
    let Some(design) = find_schematic(designs, spawn.schematic_name.get()) else {
        warn!("Skirmish references design {:?}, which is not available", spawn.schematic_name);
        return;
    };
    let team = Team::new(spawn.team);
    let skill = resolve_skill(
        spawn.ai_level,
        team,
        mission_level,
        env.setting.ai_level,
        env.ai_cfg,
    );
    env.spawn_ship(commands, design, team, spawn.position, skill);
}

pub fn hull_extent_y(design: &ShipDesign, registry: &BlockRegistry) -> Vec2 {
    let mut grid = BlockGrid::new();
    grid.rebuild_from_layout(&design.blocks, registry);

    let Some(min) = grid.cells().keys().map(|cell| cell.y).min() else {
        return Vec2::ZERO;
    };
    let Some(max) = grid.cells().keys().map(|cell| cell.y).max() else {
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

fn spawn_fleet(
    commands: &mut Commands,
    env: &SpawnEnv,
    fleet: &[ShipDesign],
    anchor: Vec2,
    spacing: f32,
) {
    let positions = formation_positions(anchor, fleet, env.registry, spacing);
    for (design, position) in fleet.iter().zip(positions) {
        let team = Team::new(DEFAULT_TEAM_ID);
        let skill = resolve_skill(None, team, None, env.setting.ai_level, env.ai_cfg);
        env.spawn_ship(commands, design, team, position, skill);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
