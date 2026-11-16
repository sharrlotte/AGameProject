use bevy::prelude::*;

use crate::content::team::{DEFAULT_TEAM_ID, Team};

use super::def::{MissionDef, ShipSpawn};
use crate::config::GameConfigValue;
use crate::content::block::BlockRegistry;
use crate::resource::DeployPayload;
use crate::save::io::{SchematicData, load_saved_schematics};

#[derive(States, Default, Clone, Copy, Eq, Hash, PartialEq, Debug)]
pub enum BattlePhase {
    #[default]
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
}

#[derive(Resource, Default)]
pub struct BattleRestart(pub bool);

pub fn find_schematic<'a>(
    schematics: &'a [SchematicData],
    name: &str,
) -> Option<&'a SchematicData> {
    schematics
        .iter()
        .find(|s| s.name == name)
        .or_else(|| schematics.first())
}

fn spawn_ship(
    commands: &mut Commands,
    registry: &BlockRegistry,
    config: &GameConfigValue,
    schematics: &[SchematicData],
    spawn: &ShipSpawn,
) {
    let Some(schematic) = find_schematic(schematics, &spawn.schematic_name) else {
        warn!(
            "Mission references schematic {:?}, which is not available",
            spawn.schematic_name
        );
        return;
    };

    schematic.to_ship_data().spawn(
        commands,
        registry,
        Team::new(spawn.team),
        Transform::from_translation(spawn.position.extend(0.0)),
        config,
    );
}

pub fn spawn_mission(
    commands: &mut Commands,
    registry: &BlockRegistry,
    config: &GameConfigValue,
    mission: &MissionDef,
    state: &DeployPayload,
) {
    let schematics = load_saved_schematics();

    match &state.current_ship {
        Some(deployed) => {
            deployed.to_ship_data().spawn(
                commands,
                registry,
                Team::new(DEFAULT_TEAM_ID),
                Transform::from_translation(mission.player.position.extend(0.0)),
                config,
            );
        }
        None => {
            spawn_ship(commands, registry, config, &schematics, &mission.player);
        }
    }

    for enemy in &mission.enemies {
        spawn_ship(commands, registry, config, &schematics, enemy);
    }
}
