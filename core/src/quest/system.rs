use std::collections::HashMap;

use bevy::asset::{Asset, AssetEvent};
use bevy::prelude::*;

use crate::config::{GameConfigValue, GameSetting};
use crate::content::block::BlockRegistry;
use crate::content::death::ShipDestroyed;
use crate::content::item::ItemCollected;
use crate::content::ship::ai::AiConfigValue;
use crate::content::ship::spawn::SpawnEnv;
use crate::content::ship::Ship;
use crate::content::team::{DEFAULT_TEAM_ID, Team};
use crate::control::{CameraTarget, ControlledShip};
use crate::quest::component::{
    QuestAcceptance, QuestAssetsReady, QuestCatalogDef, QuestCatalogValue, QuestGenConfig,
    QuestGenConfigValue, QuestHandles, QuestId, QuestObjectiveDef, QuestOfferQueue, QuestProgress,
    QuestRecord, QuestStatus, QuestTracker, VisibleOffers,
};
use crate::quest::generator::{build, validate_generated};
use crate::quest::lib::activate_available;
use crate::quest::QuestCompleted;
use crate::resource::{DeployPayload, GameRng};
use crate::save::design::ShipCatalog;
use crate::save::world::WorldContext;
use crate::scene::game::skirmish::resolve_skill;
use crate::validate::Validate;
use crate::world::SectorDiscovered;

pub fn load_quest_handles(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(QuestHandles {
        catalog: assets.load("config/quest/quests.ron"),
        generation: assets.load("config/quest/generation.ron"),
    });
}

fn apply_asset_events<A, F>(
    events: &mut MessageReader<AssetEvent<A>>,
    assets: &Assets<A>,
    mut apply: F,
) where
    A: Asset,
    F: FnMut(&A),
{
    for event in events.read() {
        let (AssetEvent::Modified { id } | AssetEvent::LoadedWithDependencies { id }) = event else {
            continue;
        };
        if let Some(value) = assets.get(*id) {
            apply(value);
        }
    }
}

pub fn load_initial_assets(
    handles: Res<QuestHandles>,
    catalogs: Res<Assets<QuestCatalogDef>>,
    configs: Res<Assets<QuestGenConfig>>,
    mut value: ResMut<QuestCatalogValue>,
    mut generation: ResMut<QuestGenConfigValue>,
    mut ready: ResMut<QuestAssetsReady>,
) {
    if ready.0 {
        return;
    }
    let Some(catalog) = catalogs.get(&handles.catalog) else {
        return;
    };
    let Some(config) = configs.get(&handles.generation) else {
        return;
    };
    apply_catalog(catalog, &mut value);
    apply_generation(config, &mut generation);
    ready.0 = true;
}

pub fn catalog_bridge(
    mut events: MessageReader<AssetEvent<QuestCatalogDef>>,
    assets: Res<Assets<QuestCatalogDef>>,
    mut value: ResMut<QuestCatalogValue>,
) {
    apply_asset_events(&mut events, &assets, |catalog| {
        apply_catalog(catalog, &mut value);
    });
}

pub fn generation_config_bridge(
    mut events: MessageReader<AssetEvent<QuestGenConfig>>,
    assets: Res<Assets<QuestGenConfig>>,
    mut value: ResMut<QuestGenConfigValue>,
) {
    apply_asset_events(&mut events, &assets, |config| {
        apply_generation(config, &mut value);
    });
}

pub fn apply_catalog(catalog: &QuestCatalogDef, value: &mut QuestCatalogValue) {
    if let Err(error) = catalog.validate(&()) {
        warn!("Quest catalog failed validation: {error}");
        return;
    }

    let mut next = QuestCatalogValue::default();
    for def in &catalog.quests {
        next.0.insert(def.id.clone(), def.clone());
    }
    for (id, def) in &value.0 {
        if id.is_generated() {
            next.0.insert(id.clone(), def.clone());
        }
    }
    *value = next;
}

fn apply_generation(config: &QuestGenConfig, value: &mut QuestGenConfigValue) {
    if let Err(error) = config.validate(&()) {
        warn!("Quest generation config failed validation: {error}");
        return;
    }
    *value = QuestGenConfigValue(config.clone());
}

pub fn activate_quests(catalog: Res<QuestCatalogValue>, mut tracker: ResMut<QuestTracker>) {
    activate_available(&mut tracker, &catalog);
}

pub fn update_quest_objectives(
    mut destroyed: MessageReader<ShipDestroyed>,
    mut collected: MessageReader<ItemCollected>,
    ships: Query<(&Transform, &Team), With<Ship>>,
    catalog: Res<QuestCatalogValue>,
    context: Res<WorldContext>,
    mut tracker: ResMut<QuestTracker>,
    mut completed: MessageWriter<QuestCompleted>,
) {
    if !context.is_save_slot() {
        return;
    }

    let kills = destroyed
        .read()
        .filter(|event| !event.team.is_player())
        .count() as u32;

    let pickups: Vec<(String, u32)> = collected
        .read()
        .filter(|event| event.team.is_player())
        .map(|event| (event.item.clone(), event.amount))
        .collect();

    let positions: Vec<Vec2> = ships
        .iter()
        .filter(|(_, team)| team.is_player())
        .map(|(transform, _)| transform.translation.truncate())
        .collect();

    if kills == 0 && pickups.is_empty() && positions.is_empty() {
        return;
    }

    let mut completed_ids = Vec::new();

    for id in tracker.active_ids() {
        let Some(def) = catalog.0.get(&id).cloned() else {
            continue;
        };
        let Some(record) = tracker.records.get_mut(&id) else {
            continue;
        };

        for objective in &def.objectives {
            let target = objective.target();
            let current = record.objective_progress_mut(objective.id());
            if *current >= target {
                continue;
            }

            match objective {
                QuestObjectiveDef::DestroyEnemies { .. } => {
                    *current = current.saturating_add(kills).min(target);
                }
                QuestObjectiveDef::CollectItem { item, .. } => {
                    let amount = pickups
                        .iter()
                        .filter(|(name, _)| name == item)
                        .map(|(_, amount)| *amount)
                        .sum::<u32>();
                    *current = current.saturating_add(amount).min(target);
                }
                QuestObjectiveDef::ReachPosition {
                    position, radius, ..
                } => {
                    if positions
                        .iter()
                        .any(|player| player.distance(*position) <= *radius)
                    {
                        *current = target;
                    }
                }
            }
        }

        let done = def
            .objectives
            .iter()
            .all(|objective| record.objective_progress(objective.id()) >= objective.target());
        if done {
            record.progress.status = QuestStatus::Completed;
            completed_ids.push(id.clone());
        }
    }

    for id in &completed_ids {
        completed.write(QuestCompleted { quest_id: id.clone() });
    }
    if !completed_ids.is_empty() {
        activate_available(&mut tracker, &catalog);
    }
}

pub fn generate_from_discovery(
    mut events: MessageReader<SectorDiscovered>,
    config: Res<QuestGenConfigValue>,
    mut catalog: ResMut<QuestCatalogValue>,
    mut tracker: ResMut<QuestTracker>,
    mut rng: ResMut<GameRng>,
    context: Res<WorldContext>,
) {
    if !context.is_save_slot() {
        for _ in events.read() {}
        return;
    }

    for event in events.read() {
        let id = QuestId::Chunk(event.chunk);
        if catalog.0.contains_key(&id) || tracker.contains(&id) {
            continue;
        }
        if tracker.count_generated_active() >= config.0.max_active_generated {
            continue;
        }

        let def = build(event.chunk, &config.0, &mut rng);
        if let Err(error) = validate_generated(&def) {
            warn!("Generated quest rejected for chunk {:?}: {error}", event.chunk);
            continue;
        }
        let status = if def.acceptance == QuestAcceptance::Auto {
            QuestStatus::Active
        } else {
            QuestStatus::Offered
        };
        let record = QuestRecord {
            def: def.clone(),
            progress: QuestProgress {
                quest_id: id.clone(),
                status,
                objective_counts: HashMap::new(),
            },
        };
        catalog.0.insert(id, def);
        tracker.insert(record);
    }
}

pub fn sync_offer_visibility(
    ships: Query<(&Transform, &Team), With<Ship>>,
    tracker: Res<QuestTracker>,
    context: Res<WorldContext>,
    mut visible: ResMut<VisibleOffers>,
    mut queue: ResMut<QuestOfferQueue>,
) {
    if !context.is_save_slot() {
        visible.0.clear();
        queue.pending.clear();
        queue.open = None;
        return;
    }

    let positions: Vec<Vec2> = ships
        .iter()
        .filter(|(_, team)| team.is_player())
        .map(|(transform, _)| transform.translation.truncate())
        .collect();

    let mut next = Vec::new();

    for (id, record) in &tracker.records {
        if record.progress.status != QuestStatus::Offered {
            continue;
        }
        let Some(offer) = record.def.offer.as_ref() else {
            continue;
        };
        if positions
            .iter()
            .any(|player| player.distance(offer.position) <= offer.radius)
        {
            next.push(id.clone());
        }
    }
    next.sort_by_key(QuestId::key);

    if let Some(open) = &queue.open
        && !next.contains(open)
    {
        queue.open = None;
    }
    queue.pending = next.clone();
    visible.0 = next;
}

pub fn respawn_player_on_defeat(
    mut events: MessageReader<ShipDestroyed>,
    ships: Query<(Entity, &Team), With<Ship>>,
    mut commands: Commands,
    registry: Res<BlockRegistry>,
    config: Res<GameConfigValue>,
    fleet: Res<DeployPayload>,
    catalog: Res<ShipCatalog>,
    setting: Res<GameSetting>,
    ai_cfg: Res<AiConfigValue>,
    mut controlled: ResMut<ControlledShip>,
    mut camera_target: ResMut<CameraTarget>,
) {
    let player_destroyed = events
        .read()
        .any(|event| event.team.is_player());

    if !respawn_needed(player_destroyed, ships.iter().any(|(_, team)| team.is_player())) {
        return;
    }

    let Some(design) = fleet
        .fleet
        .first()
        .or_else(|| catalog.designs().first())
        .cloned()
    else {
        warn!("Respawn failed: no starter design available");
        return;
    };

    let team = Team::new(DEFAULT_TEAM_ID);
    let skill = resolve_skill(None, team, None, setting.ai_level, &ai_cfg.0);
    let env = SpawnEnv::new(&registry, &config, &setting, &ai_cfg.0);
    
    if let Some(entity) = env.spawn_ship(&mut commands, &design, team, Vec2::ZERO, skill) {
        controlled.controlling = Some(entity);
        camera_target.following = Some(entity);
    }
}

fn respawn_needed(player_destroyed: bool, any_player_alive: bool) -> bool {
    player_destroyed && !any_player_alive
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AiConfigValue, GameConfig, GameSetting};
    use crate::content::block::BlockRegistry;
    use crate::content::ship::ai::AiConfig;
    use crate::quest::component::{QuestDef, QuestGenConfig, QuestOffer, QuestPrereqMode};
    use crate::quest::lib::{activate_available, prerequisites_met};
    use crate::resource::DeployPayload;
    use crate::save::design::ShipCatalog;
    use crate::save::world::store::InMemoryWorldStore;
    use crate::world::SectorDiscovered;

    fn sample(id: &QuestId) -> QuestDef {
        QuestDef {
            id: id.clone(),
            title: "sample".to_string(),
            description: String::new(),
            requires: Vec::new(),
            requires_mode: Default::default(),
            acceptance: Default::default(),
            offer: None,
            objectives: vec![QuestObjectiveDef::DestroyEnemies {
                id: "kills".to_string(),
                count: 1,
            }],
        }
    }

    fn minimal_app(context: WorldContext) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<ShipDestroyed>()
            .add_message::<ItemCollected>()
            .add_message::<QuestCompleted>()
            .add_message::<SectorDiscovered>()
            .insert_resource(QuestCatalogValue::default())
            .insert_resource(QuestTracker::default())
            .insert_resource(QuestGenConfigValue(QuestGenConfig::default()))
            .insert_resource(GameRng::new(1))
            .insert_resource(context)
            .insert_resource(VisibleOffers::default())
            .insert_resource(QuestOfferQueue::default())
            .insert_resource(BlockRegistry::default())
            .insert_resource(GameConfigValue(GameConfig::default()))
            .insert_resource(DeployPayload::default())
            .insert_resource(ShipCatalog::default())
            .insert_resource(GameSetting::default())
            .insert_resource(AiConfigValue(AiConfig::default()))
            .init_resource::<ControlledShip>()
            .init_resource::<CameraTarget>();
        app
    }

    fn save_slot_context() -> WorldContext {
        WorldContext::SaveSlot {
            slot_id: uuid::Uuid::now_v7(),
            path: std::path::PathBuf::new(),
            store: Box::new(InMemoryWorldStore::default()),
        }
    }

    fn quick_battle_context() -> WorldContext {
        WorldContext::default()
    }

    fn install_quest(app: &mut App, def: QuestDef) -> QuestId {
        let id = def.id.clone();
        app.world_mut()
            .resource_mut::<QuestCatalogValue>()
            .0
            .insert(id.clone(), def);
        let catalog = app.world().resource::<QuestCatalogValue>().clone();
        activate_available(&mut app.world_mut().resource_mut::<QuestTracker>(), &catalog);
        id
    }

    #[test]
    fn started_prerequisite_unlocks_while_predecessor_is_active() {
        let mut tracker = QuestTracker::default();
        let first = QuestId::Authored("first".to_string());
        tracker.insert(QuestRecord {
            def: sample(&first),
            progress: QuestProgress {
                quest_id: first.clone(),
                status: QuestStatus::Active,
                objective_counts: HashMap::new(),
            },
        });
        let mut successor = sample(&QuestId::Authored("second".to_string()));
        successor.requires = vec![first];
        successor.requires_mode = QuestPrereqMode::Started;

        assert!(prerequisites_met(&successor, &tracker));
    }

    #[test]
    fn completed_prerequisite_waits_for_completion() {
        let mut tracker = QuestTracker::default();
        let first = QuestId::Authored("first".to_string());
        tracker.insert(QuestRecord {
            def: sample(&first),
            progress: QuestProgress {
                quest_id: first.clone(),
                status: QuestStatus::Active,
                objective_counts: HashMap::new(),
            },
        });
        let mut successor = sample(&QuestId::Authored("second".to_string()));
        successor.requires = vec![first.clone()];

        assert!(!prerequisites_met(&successor, &tracker));
        tracker.records.get_mut(&first).unwrap().progress.status = QuestStatus::Completed;
        assert!(prerequisites_met(&successor, &tracker));
    }

    #[test]
    fn enemy_kills_complete_objective_and_emit_completed() {
        let mut app = minimal_app(save_slot_context());
        let mut def = sample(&QuestId::Authored("kills".to_string()));
        def.objectives = vec![QuestObjectiveDef::DestroyEnemies {
            id: "kills".to_string(),
            count: 2,
        }];
        let id = install_quest(&mut app, def);
        app.add_systems(FixedPostUpdate, update_quest_objectives);

        app.world_mut().write_message(ShipDestroyed {
            ship: Entity::PLACEHOLDER,
            team: Team::new(1),
        });
        app.world_mut().write_message(ShipDestroyed {
            ship: Entity::PLACEHOLDER,
            team: Team::new(2),
        });
        app.world_mut().run_schedule(FixedPostUpdate);

        assert_eq!(
            app.world().resource::<QuestTracker>().status(&id),
            Some(QuestStatus::Completed)
        );
        let completed = app
            .world()
            .resource::<Messages<QuestCompleted>>()
            .iter_current_update_messages()
            .count();
        assert_eq!(completed, 1);
    }

    #[test]
    fn collect_item_progress_is_cumulative() {
        let mut app = minimal_app(save_slot_context());
        let mut def = sample(&QuestId::Authored("salvage".to_string()));
        def.objectives = vec![QuestObjectiveDef::CollectItem {
            id: "steel".to_string(),
            item: "steel".to_string(),
            count: 10,
        }];
        let id = install_quest(&mut app, def);
        app.add_systems(FixedPostUpdate, update_quest_objectives);

        app.world_mut().write_message(ItemCollected {
            item: "steel".to_string(),
            amount: 6,
            team: Team::new(0),
        });
        app.world_mut().run_schedule(FixedPostUpdate);
        app.world_mut().write_message(ItemCollected {
            item: "steel".to_string(),
            amount: 4,
            team: Team::new(0),
        });
        app.world_mut().run_schedule(FixedPostUpdate);

        let tracker = app.world().resource::<QuestTracker>();
        assert_eq!(
            tracker.records[&id].progress.objective_counts.get("steel"),
            Some(&10)
        );
        assert_eq!(tracker.status(&id), Some(QuestStatus::Completed));
    }

    #[test]
    fn reach_position_completes_when_player_is_in_radius() {
        let mut app = minimal_app(save_slot_context());
        let mut def = sample(&QuestId::Authored("reach".to_string()));
        def.objectives = vec![QuestObjectiveDef::ReachPosition {
            id: "beacon".to_string(),
            position: Vec2::new(1000.0, 0.0),
            radius: 200.0,
        }];
        let id = install_quest(&mut app, def);
        app.world_mut().spawn((
            Ship::default(),
            Team::new(0),
            Transform::from_xyz(900.0, 0.0, 0.0),
        ));
        app.add_systems(FixedPostUpdate, update_quest_objectives);

        app.world_mut().run_schedule(FixedPostUpdate);
        assert_eq!(
            app.world().resource::<QuestTracker>().status(&id),
            Some(QuestStatus::Completed)
        );
    }

    #[test]
    fn reordering_objectives_preserves_progress_by_stable_id() {
        let id = QuestId::Authored("reorder".to_string());
        let mut first = sample(&id);
        first.objectives = vec![
            QuestObjectiveDef::DestroyEnemies {
                id: "kills".to_string(),
                count: 3,
            },
            QuestObjectiveDef::CollectItem {
                id: "steel".to_string(),
                item: "steel".to_string(),
                count: 3,
            },
        ];

        let mut tracker = QuestTracker::default();
        tracker.insert(QuestRecord {
            def: first.clone(),
            progress: QuestProgress {
                quest_id: id.clone(),
                status: QuestStatus::Active,
                objective_counts: HashMap::from([("kills".to_string(), 2), ("steel".to_string(), 1)]),
            },
        });

        let mut reordered = first;
        reordered.objectives.reverse();
        let catalog = QuestCatalogValue(HashMap::from([(id.clone(), reordered)]));
        activate_available(&mut tracker, &catalog);

        let record = &tracker.records[&id];
        assert_eq!(record.progress.objective_counts["kills"], 2);
        assert_eq!(record.progress.objective_counts["steel"], 1);
    }

    #[test]
    fn skirmish_kills_do_not_advance_saved_quests() {
        let mut app = minimal_app(quick_battle_context());
        let mut def = sample(&QuestId::Authored("kills".to_string()));
        def.objectives = vec![QuestObjectiveDef::DestroyEnemies {
            id: "kills".to_string(),
            count: 2,
        }];
        let id = install_quest(&mut app, def);
        app.add_systems(FixedPostUpdate, update_quest_objectives);

        for _ in 0..2 {
            app.world_mut().write_message(ShipDestroyed {
                ship: Entity::PLACEHOLDER,
                team: Team::new(1),
            });
        }
        app.world_mut().run_schedule(FixedPostUpdate);

        let tracker = app.world().resource::<QuestTracker>();
        assert_eq!(tracker.status(&id), Some(QuestStatus::Active));
        assert_eq!(
            tracker.records[&id].progress.objective_counts.get("kills"),
            None
        );
    }

    #[test]
    fn generation_dedupes_by_chunk_and_honours_cap() {
        let mut app = minimal_app(save_slot_context());
        app.add_systems(FixedPostUpdate, generate_from_discovery);
        app.world_mut().write_message(SectorDiscovered {
            position: Vec2::ZERO,
            chunk: IVec2::new(1, 1),
        });
        app.world_mut().write_message(SectorDiscovered {
            position: Vec2::ZERO,
            chunk: IVec2::new(1, 1),
        });
        app.world_mut().run_schedule(FixedPostUpdate);
        assert_eq!(
            app.world().resource::<QuestTracker>().records.len(),
            1
        );

        app.world_mut()
            .resource_mut::<QuestGenConfigValue>()
            .0
            .max_active_generated = 0;
        app.world_mut().write_message(SectorDiscovered {
            position: Vec2::ZERO,
            chunk: IVec2::new(2, 2),
        });
        app.world_mut().run_schedule(FixedPostUpdate);
        assert_eq!(
            app.world().resource::<QuestTracker>().records.len(),
            1
        );
    }

    #[test]
    fn generation_is_dormant_in_quick_battle() {
        let mut app = minimal_app(quick_battle_context());
        app.add_systems(FixedPostUpdate, generate_from_discovery);
        app.world_mut().write_message(SectorDiscovered {
            position: Vec2::ZERO,
            chunk: IVec2::new(3, 3),
        });
        app.world_mut().run_schedule(FixedPostUpdate);
        assert!(app
            .world()
            .resource::<QuestTracker>()
            .records
            .is_empty());
    }

    #[test]
    fn offer_visibility_follows_player_range() {
        let mut app = minimal_app(save_slot_context());
        let id = QuestId::Chunk(IVec2::new(0, 0));
        let mut def = sample(&id);
        def.acceptance = QuestAcceptance::Manual;
        def.offer = Some(QuestOffer {
            position: Vec2::ZERO,
            radius: 100.0,
        });
        let mut tracker = app.world_mut().remove_resource::<QuestTracker>().unwrap();
        tracker.insert(QuestRecord {
            def,
            progress: QuestProgress {
                quest_id: id.clone(),
                status: QuestStatus::Offered,
                objective_counts: HashMap::new(),
            },
        });
        app.world_mut().insert_resource(tracker);
        let ship = app
            .world_mut()
            .spawn((
                Ship::default(),
                Team::new(0),
                Transform::from_xyz(10.0, 0.0, 0.0),
            ))
            .id();
        app.add_systems(FixedPostUpdate, sync_offer_visibility);

        app.world_mut().run_schedule(FixedPostUpdate);
        assert_eq!(app.world().resource::<VisibleOffers>().0, vec![id.clone()]);

        app.world_mut()
            .entity_mut(ship)
            .insert(Transform::from_xyz(1000.0, 0.0, 0.0));
        app.world_mut().run_schedule(FixedPostUpdate);
        assert!(app.world().resource::<VisibleOffers>().0.is_empty());
        assert_eq!(
            app.world().resource::<QuestTracker>().status(&id),
            Some(QuestStatus::Offered)
        );
    }

    #[test]
    fn respawn_condition_ignores_enemy_deaths_and_living_allies() {
        assert!(!respawn_needed(false, false));
        assert!(!respawn_needed(true, true));
        assert!(respawn_needed(true, false));
    }

}
