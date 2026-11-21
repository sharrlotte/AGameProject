use bevy::prelude::*;

use crate::quest::component::{
    QuestCatalogValue, QuestDef, QuestId, QuestPrereqMode, QuestProgress, QuestRecord, QuestStatus,
    QuestTracker,
};
use crate::save::world::component::QuestStoreRow;
use crate::save::world::store::{decode, encode};

pub fn prerequisites_met(def: &QuestDef, tracker: &QuestTracker) -> bool {
    def.requires.iter().all(|required| match def.requires_mode {
        QuestPrereqMode::Completed => tracker.status(required) == Some(QuestStatus::Completed),
        QuestPrereqMode::Started => matches!(
            tracker.status(required),
            Some(QuestStatus::Active | QuestStatus::Completed)
        ),
    })
}

pub fn activate_available(tracker: &mut QuestTracker, catalog: &QuestCatalogValue) {
    let locked: Vec<QuestId> = catalog
        .0
        .keys()
        .filter(|id| !tracker.contains(id))
        .filter(|id| {
            catalog
                .0
                .get(*id)
                .is_some_and(|def| prerequisites_met(def, tracker))
        })
        .cloned()
        .collect();

    for id in locked {
        let Some(def) = catalog.0.get(&id) else {
            continue;
        };
        let record = QuestRecord {
            def: def.clone(),
            progress: QuestProgress {
                quest_id: id,
                status: QuestStatus::Active,
                objective_counts: Default::default(),
            },
        };
        tracker.insert(record);
    }
}

pub fn quest_rows(tracker: &QuestTracker) -> Vec<QuestStoreRow> {
    tracker
        .records
        .values()
        .filter(|record| record.progress.status != QuestStatus::Offered)
        .filter_map(|record| match encode(record) {
            Ok(blob) => Some(QuestStoreRow {
                quest_id: record.progress.quest_id.key(),
                status: record.progress.status,
                record_blob: blob,
            }),
            Err(error) => {
                warn!("Failed to encode quest {}: {error}", record.progress.quest_id.key());
                None
            }
        })
        .collect()
}

pub fn restore_tracker(rows: &[QuestStoreRow], catalog: &QuestCatalogValue) -> QuestTracker {
    let mut tracker = QuestTracker::default();
    for row in rows {
        match decode::<QuestRecord>(&row.record_blob) {
            Ok(mut record) => {
                if let QuestId::Authored(_) = record.progress.quest_id
                    && let Some(def) = catalog.0.get(&record.progress.quest_id)
                {
                    record.def = def.clone();
                }
                tracker.insert(record);
            }
            Err(error) => {
                warn!("Failed to decode saved quest {}: {error}", row.quest_id);
            }
        }
    }
    tracker
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::quest::component::{QuestAcceptance, QuestId, QuestProgress};
    use crate::quest::component::{QuestDef, QuestObjectiveDef};
    use crate::save::world::component::{ActiveStateSnapshot, QuestStoreRow, WorldMeta};
    use crate::save::world::store::{InMemoryWorldStore, WorldStore};

    fn record(id: QuestId, status: QuestStatus) -> QuestRecord {
        let def = QuestDef {
            id: id.clone(),
            title: format!("title-{}", id.key()),
            description: String::new(),
            requires: Vec::new(),
            requires_mode: Default::default(),
            acceptance: QuestAcceptance::default(),
            offer: None,
            objectives: vec![QuestObjectiveDef::DestroyEnemies {
                id: "kills".to_string(),
                count: 3,
            }],
        };
        QuestRecord {
            def,
            progress: QuestProgress {
                quest_id: id,
                status,
                objective_counts: HashMap::from([("kills".to_string(), 1)]),
            },
        }
    }

    #[test]
    fn offered_quests_are_not_persisted() {
        let mut tracker = QuestTracker::default();
        tracker.insert(record(
            QuestId::Authored("active".to_string()),
            QuestStatus::Active,
        ));
        tracker.insert(record(
            QuestId::Authored("completed".to_string()),
            QuestStatus::Completed,
        ));
        tracker.insert(record(QuestId::Chunk(IVec2::new(1, 2)), QuestStatus::Offered));

        let rows = quest_rows(&tracker);
        assert_eq!(rows.len(), 2);
        assert!(rows
            .iter()
            .all(|row| row.status != QuestStatus::Offered));
    }

    #[test]
    fn quest_records_round_trip_through_world_store() {
        let mut tracker = QuestTracker::default();
        tracker.insert(record(
            QuestId::Authored("active".to_string()),
            QuestStatus::Active,
        ));
        tracker.insert(record(QuestId::Chunk(IVec2::new(4, -5)), QuestStatus::Completed));
        let rows = quest_rows(&tracker);

        let store = InMemoryWorldStore::default();
        let meta = WorldMeta {
            name: "World".to_string(),
            seed: 1,
            play_time_secs: 0.0,
            created_at_ms: 0,
            updated_at_ms: 0,
            camera_blob: Vec::new(),
            control_blob: Vec::new(),
        };
        store
            .save_world(&meta, &ActiveStateSnapshot::default(), &rows)
            .expect("save world");
        let loaded: Vec<QuestStoreRow> = store.load_quests().expect("load quests");

        let restored = restore_tracker(&loaded, &QuestCatalogValue::default());
        assert_eq!(restored.records.len(), 2);
        assert_eq!(
            restored.status(&QuestId::Authored("active".to_string())),
            Some(QuestStatus::Active)
        );
        assert_eq!(
            restored.status(&QuestId::Chunk(IVec2::new(4, -5))),
            Some(QuestStatus::Completed)
        );
    }

    #[test]
    fn authored_definition_authority_uses_catalog() {
        let id = QuestId::Authored("tutorial".to_string());
        let mut saved = record(id.clone(), QuestStatus::Active);
        saved.def.title = "old title".to_string();
        let blob = encode(&saved).expect("encode");
        let rows = vec![QuestStoreRow {
            quest_id: id.key(),
            status: QuestStatus::Active,
            record_blob: blob,
        }];

        let mut catalog = QuestCatalogValue::default();
        let mut fresh = saved.def.clone();
        fresh.title = "new title".to_string();
        catalog.0.insert(id.clone(), fresh);

        let restored = restore_tracker(&rows, &catalog);
        assert_eq!(restored.records[&id].def.title, "new title");
    }

    #[test]
    fn generated_definition_authority_uses_saved_blob() {
        let id = QuestId::Chunk(IVec2::new(9, 9));
        let mut saved = record(id.clone(), QuestStatus::Active);
        saved.def.title = "generated title".to_string();
        let blob = encode(&saved).expect("encode");
        let rows = vec![QuestStoreRow {
            quest_id: id.key(),
            status: QuestStatus::Active,
            record_blob: blob,
        }];

        let mut catalog = QuestCatalogValue::default();
        let mut competing = saved.def.clone();
        competing.title = "catalog title".to_string();
        catalog.0.insert(id.clone(), competing);

        let restored = restore_tracker(&rows, &catalog);
        assert_eq!(restored.records[&id].def.title, "generated title");
    }
}
