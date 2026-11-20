use std::collections::HashMap;

use bevy::prelude::*;
use bevy_rapier2d::prelude::Velocity;
use uuid::Uuid;

use crate::config::GameConfigValue;
use crate::constant::LAYER_DROP;

use crate::content::block::{
    BlockPlacement, BlockRegistry, ItemDrop, Loot, spawn_block_entity,
};
use crate::content::body::debris::{component_mass, spawn_debris_body};
use crate::content::body::BlockGrid;
use crate::content::item::ItemRegistry;
use crate::content::lifetime::Lifetime;
use crate::content::ship::ai::{Brain, LastSeen, Skill, TacticalBoard, Tactic, TeamAlert, TeamPlan};
use crate::content::ship::stat::ShipStats;
use crate::content::ship::ShipDesign;
use crate::content::team::Team;
use crate::resource::{CameraTarget, ControlMode, ControlledShip, SceneState};
use crate::save::design::DESIGN_SCHEMA_VERSION;
use crate::save::world::component::WorldStoreError;

use super::BodySnapshot;
use super::{
    BrainSave, CameraSave, ControlSave, DebrisSnapshot, DropSave, ShipSnapshot, TacticSave,
    TeamIntelSave,
};

impl TacticSave {
    pub fn into_runtime(self, ship_map: &HashMap<Uuid, Entity>, fallback: Vec2) -> Tactic {
        let res = |id: Option<Uuid>| id.and_then(|uuid| ship_map.get(&uuid).copied());
        match self {
            Self::Engage { target, range, posture } => res(target).map_or(
                Tactic::Search { waypoint: fallback },
                |target| Tactic::Engage { target, range, posture },
            ),
            Self::Recharge { threat, safe_range } => res(threat).map_or(
                Tactic::Search { waypoint: fallback },
                |threat| Tactic::Recharge { threat, safe_range },
            ),
            Self::Retreat { threat } => res(threat).map_or(
                Tactic::Search { waypoint: fallback },
                |threat| Tactic::Retreat { threat },
            ),
            Self::Support { ally, kind } => res(ally).map_or(
                Tactic::Search { waypoint: fallback },
                |ally| Tactic::Support { ally, kind },
            ),
            Self::Regroup { position } => Tactic::Regroup { position },
            Self::Investigate { position } => Tactic::Investigate { position },
            Self::Search { waypoint } => Tactic::Search { waypoint },
        }
    }
}

impl ShipSnapshot {
    pub fn restore(
        self,
        commands: &mut Commands,
        registry: &BlockRegistry,
        config: &GameConfigValue,
    ) -> Result<(Uuid, Entity, u8, BrainSave), WorldStoreError> {
        let placements: Vec<BlockPlacement> =
            self.blocks.iter().map(|b| b.placement.clone()).collect();
        let design = ShipDesign {
            version: DESIGN_SCHEMA_VERSION,
            id: self.id,
            name: String::new(),
            blocks: placements,
            doctrine: self.doctrine,
        };
        let team = Team::new(self.team);
        let transform = Transform::from_xyz(self.transform.x, self.transform.y, 0.0)
            .with_rotation(Quat::from_rotation_z(self.transform.rotation));

        let Ok((ship, block_entities)) =
            design.spawn_with_id(commands, registry, team, transform, config, self.id)
        else {
            return Err(WorldStoreError::InvalidData(format!(
                "saved ship {} has an invalid layout",
                self.id
            )));
        };

        commands.entity(ship).insert(Velocity {
            linear: Vec2::new(self.velocity.x, self.velocity.y),
            angular: self.velocity.angvel,
        });

        let cargo = self.cargo;
        commands.entity(ship).queue(move |mut entity: EntityWorldMut| {
            if let Some(mut stats) = entity.get_mut::<ShipStats>() {
                let derived: u32 = cargo.items.values().sum();
                if derived != cargo.used {
                    warn!("cargo used {} disagrees with manifest total {}; deriving from manifest", cargo.used, derived);
                }
                stats.set_cargo_items(cargo.items.into_iter().collect());
            }
        });

        for (entity, snapshot) in block_entities.iter().zip(self.blocks) {
            snapshot.apply(commands, *entity);
        }

        Ok((self.id, ship, self.skill, self.brain))
    }
}

impl DebrisSnapshot {
    pub fn restore(
        self,
        commands: &mut Commands,
        registry: &BlockRegistry,
        config: &GameConfigValue,
    ) -> Entity {
        let placements: Vec<BlockPlacement> =
            self.blocks.iter().map(|b| b.placement.clone()).collect();
        let mut grid = BlockGrid::new();
        grid.rebuild_from_layout(&placements, registry);

        let transform = Transform::from_xyz(self.transform.x, self.transform.y, 0.0)
            .with_rotation(Quat::from_rotation_z(self.transform.rotation));
        let velocity = Velocity {
            linear: Vec2::new(self.velocity.x, self.velocity.y),
            angular: self.velocity.angvel,
        };
        let team = Team::new(self.team);
        let collider = registry.build_voxel_collider_2d(&grid);
        let mass = component_mass(&grid, registry);

        let body = spawn_debris_body(
            commands,
            grid,
            transform,
            velocity,
            team,
            self.remaining_secs,
            collider,
            mass,
            config,
        );

        let mut spawned = Vec::with_capacity(self.blocks.len());
        for snapshot in self.blocks {
            let Some(def) = registry.get_symbol(&snapshot.placement.block_type) else {
                continue;
            };
            let id = snapshot.placement.id;
            let block = spawn_block_entity(commands, &snapshot.placement, def, body, team, config);
            snapshot.apply(commands, block);
            spawned.push((id, block));
        }

        commands.entity(body).queue(move |mut entity: EntityWorldMut| {
            if let Some(mut grid) = entity.get_mut::<BlockGrid>() {
                for (id, block) in spawned {
                    grid.set_entity(id, block);
                }
            }
        });

        body
    }
}

pub fn apply_bodies(
    commands: &mut Commands,
    bodies: Vec<BodySnapshot>,
    registry: &BlockRegistry,
    config: &GameConfigValue,
) -> Result<HashMap<Uuid, Entity>, WorldStoreError> {
    let (ship_bodies, debris_bodies): (Vec<BodySnapshot>, Vec<BodySnapshot>) = bodies
        .into_iter()
        .partition(|body| matches!(body, BodySnapshot::Ship(_)));

    let mut ship_map = HashMap::with_capacity(ship_bodies.len());
    let mut pending_ai = Vec::with_capacity(ship_bodies.len());

    for body in ship_bodies {
        let BodySnapshot::Ship(ship) = body else { continue };
        let (id, entity, skill, brain) = ship.restore(commands, registry, config)?;
        ship_map.insert(id, entity);
        pending_ai.push((entity, skill, brain));
    }

    for body in debris_bodies {
        let BodySnapshot::Debris(debris) = body else { continue };
        debris.restore(commands, registry, config);
    }

    for (entity, skill, brain) in pending_ai {
        let tactic = brain.current.into_runtime(&ship_map, brain.spawn_position);
        let duration = brain.timer_secs.max(0.05);
        let timer = Timer::from_seconds(duration, TimerMode::Repeating);
        commands.entity(entity).insert((
            Skill::new(skill),
            Brain {
                current: tactic,
                spawn_position: brain.spawn_position,
                think_timer: timer,
                strafe_sign: if brain.strafe_sign == 0.0 { 1.0 } else { brain.strafe_sign },
                strafe_timer: brain.strafe_timer.max(0.1),
            },
        ));
    }

    Ok(ship_map)
}

pub fn apply_intel(
    commands: &mut Commands,
    intel: Vec<TeamIntelSave>,
    ship_map: &HashMap<Uuid, Entity>,
) {
    let mut board = TacticalBoard::default();
    for saved_intel in intel {
        let mut plan = TeamPlan::default();
        for seen in saved_intel.last_known {
            if let Some(&target) = ship_map.get(&seen.target_id) {
                plan.intel.last_known.insert(
                    target,
                    LastSeen { position: seen.position, velocity: seen.velocity, ttl: seen.ttl },
                );
            }
        }
        for alert in saved_intel.alerts {
            plan.intel.alerts.push(TeamAlert { position: alert.position, ttl: alert.ttl });
        }
        board.teams.insert(saved_intel.team, plan);
    }
    commands.insert_resource(board);
}

pub fn apply_control(commands: &mut Commands, control: &ControlSave, ship_map: &HashMap<Uuid, Entity>) {
    let controlling = control.controlling.and_then(|id| ship_map.get(&id).copied());
    let mode = if control.pilot_mode && controlling.is_some() {
        ControlMode::Pilot
    } else {
        ControlMode::FreeLook
    };
    commands.insert_resource(ControlledShip { controlling });
    commands.insert_resource(mode);
}

pub fn apply_camera(
    camera: &mut Query<(&mut Transform, &mut Projection), With<Camera2d>>,
    saved: &CameraSave,
    config: &GameConfigValue,
    ship_map: &HashMap<Uuid, Entity>,
    commands: &mut Commands,
) {
    let following = saved.following.and_then(|id| ship_map.get(&id).copied());
    commands.insert_resource(CameraTarget { following });

    for (mut transform, mut projection) in camera {
        transform.translation.x = saved.x;
        transform.translation.y = saved.y;
        if let Projection::Orthographic(ortho) = &mut *projection {
            ortho.scale = saved.zoom.clamp(config.0.min_zoom, config.0.max_zoom);
        }
    }
}

pub fn apply_drops(
    commands: &mut Commands,
    drops: Vec<DropSave>,
    config: &GameConfigValue,
    items: Option<&ItemRegistry>,
) {
    for drop in drops {
        let color = items
            .and_then(|reg| reg.get(&drop.item_type))
            .map(|def| def.rarity.color())
            .unwrap_or(Color::srgb(5.0, 5.0, 5.0));
        commands.spawn((
            Loot::Item(ItemDrop { item_type: drop.item_type, amount: drop.amount }),
            DespawnOnExit(SceneState::Game),
            Transform::from_translation(drop.position.extend(LAYER_DROP)),
            Sprite {
                color,
                custom_size: Some(Vec2::new(config.0.drop_size, config.0.drop_size)),
                ..default()
            },
            Lifetime::seconds(drop.remaining_secs.max(0.0)),
        ));
    }
}
