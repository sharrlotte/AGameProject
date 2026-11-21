use bevy::prelude::*;

use crate::config::{GameConfig, GameSettings};
use crate::content::block::BlockRegistry;
use crate::content::ship::ai::{AiConfig, Brain, Skill};
use crate::content::ship::ShipDesign;
use crate::content::team::Team;

pub struct SpawnEnv<'a> {
    pub registry: &'a BlockRegistry,
    pub config: &'a GameConfig,
    pub setting: &'a GameSettings,
    pub ai_cfg: &'a AiConfig,
}

impl<'a> SpawnEnv<'a> {
    pub fn new(
        registry: &'a BlockRegistry,
        config: &'a GameConfig,
        setting: &'a GameSettings,
        ai_cfg: &'a AiConfig,
    ) -> Self {
        Self {
            registry,
            config,
            setting,
            ai_cfg,
        }
    }

    pub fn spawn_ship(
        &self,
        commands: &mut Commands,
        design: &ShipDesign,
        team: Team,
        position: Vec2,
        skill: Skill,
    ) -> Option<Entity> {
        let think_hz = self.ai_cfg.curves.think_rate_hz.sample(skill.as_skill_level());
        match design.spawn(
            commands,
            self.registry,
            team,
            Transform::from_translation(position.extend(0.0)),
            self.config,
        ) {
            Ok((entity, _)) => {
                commands
                    .entity(entity)
                    .insert((skill, Brain::new(position, think_hz)));
                Some(entity)
            }
            Err(errors) => {
                warn!("Ship spawn failed for {:?}: {errors:?}", design.name);
                None
            }
        }
    }
}
