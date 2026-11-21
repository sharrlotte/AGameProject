use bevy::asset::LoadState;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use serde::{Deserialize, Serialize};

use crate::config::loader::RonAssetLoader;
use crate::config::{ConfigState, GameConfigValue, GameSetting};
use crate::constant::GRID_SIZE;
use crate::content::block::BlockRegistry;
use crate::content::body::BlockGrid;
use crate::content::ship::ai::{AiConfig, AiConfigValue, Skill};
use crate::content::ship::spawn::SpawnEnv;
use crate::content::ship::ShipDesign;
use crate::content::team::{DEFAULT_TEAM_ID, Team};
use crate::resource::{AppNotice, DeployPayload, SceneState};
use crate::save::design::ShipCatalog;
use crate::save::world::WorldContext;
use crate::simulation::SimulationSet;
use crate::validate::{Validate, check};

#[derive(Asset, TypePath, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct SkirmishDef {
    #[serde(default)]
    pub ai_level: Option<u8>,
    pub spawn: Vec2,
    pub enemies: Vec<ShipSpawn>,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Debug)]
pub struct ShipSpawn {
    pub team: u8,
    pub position: Vec2,
    pub schematic_name: String,
    #[serde(default)]
    pub ai_level: Option<u8>,
}

impl Validate for SkirmishDef {
    type Context = ();
    type Error = String;

    fn validate(&self, _: &()) -> Result<(), String> {
        if let Some(level) = self.ai_level {
            check::bounded_int("skirmish AI level", level, 1, 10)?;
        }
        for enemy in &self.enemies {
            if enemy.schematic_name.trim().is_empty() {
                return Err("enemy schematic name must not be empty".to_string());
            }
        }
        Ok(())
    }
}

#[derive(Resource)]
struct SkirmishAsset(Handle<SkirmishDef>);

#[derive(Resource, Default)]
struct SkirmishPending(bool);

pub struct SkirmishPlugin;

impl Plugin for SkirmishPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<SkirmishDef>()
            .register_asset_loader(RonAssetLoader::<SkirmishDef>::new(&["ron"]))
            .init_resource::<SkirmishPending>()
            .add_systems(Startup, load_skirmish)
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

fn load_skirmish(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(SkirmishAsset(
        assets.load("config/base/skirmish.ron"),
    ));
}

fn begin_skirmish(context: Res<WorldContext>, mut pending: ResMut<SkirmishPending>) {
    pending.0 = !context.is_restoring();
}

#[allow(clippy::too_many_arguments)]
fn spawn_skirmish(
    mut commands: Commands,
    assets: Res<Assets<SkirmishDef>>,
    server: Res<AssetServer>,
    handle: Option<Res<SkirmishAsset>>,
    registry: Res<BlockRegistry>,
    config: Res<GameConfigValue>,
    setting: Res<GameSetting>,
    ai_cfg: Res<AiConfigValue>,
    fleet: Res<DeployPayload>,
    catalog: Res<ShipCatalog>,
    mut notice: ResMut<AppNotice>,
    mut pending: ResMut<SkirmishPending>,
) {
    let Some(handle) = handle else {
        return;
    };

    if matches!(server.get_load_state(&handle.0), Some(LoadState::Failed(_))) {
        warn!("config/base/skirmish.ron failed to load");
        notice.show("Skirmish failed to load; returning home.");
        pending.0 = false;
        return;
    }
    let Some(def) = assets.get(&handle.0).cloned() else {
        return;
    };

    let env = SpawnEnv::new(&registry, &config, &setting, &ai_cfg.0);
    spawn_fleet(&mut commands, &env, &fleet.fleet, def.spawn, config.0.fleet_spacing);
    for enemy in &def.enemies {
        spawn_enemy(&mut commands, &env, catalog.designs(), enemy, def.ai_level);
    }

    commands.remove_resource::<SkirmishAsset>();
    pending.0 = false;
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
    mission_level: Option<u8>,
) {
    let Some(design) = find_schematic(designs, &spawn.schematic_name) else {
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
