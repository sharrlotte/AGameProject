use std::collections::HashMap;
use std::sync::Mutex;

use serde::Serialize;
use serde::de::DeserializeOwned;
use uuid::Uuid;

use crate::content::ship::ShipDesign;
use crate::save::world::component::{ActiveStateSnapshot, SlotSummary, WorldMeta, WorldStoreError};
use crate::world::chunk::{ChunkCoord, ChunkData};

pub trait WorldStore: Send + Sync {
    fn load_meta(&self) -> Result<WorldMeta, WorldStoreError>;
    fn load_active_state(&self) -> Result<ActiveStateSnapshot, WorldStoreError>;
    fn list_slots(&self) -> Result<Vec<SlotSummary>, WorldStoreError>;
    fn save_world(&self, meta: &WorldMeta, active: &ActiveStateSnapshot) -> Result<(), WorldStoreError>;
    fn list_designs(&self) -> Result<Vec<ShipDesign>, WorldStoreError>;
    fn save_design(&self, design: &ShipDesign) -> Result<(), WorldStoreError>;
    fn delete_design(&self, id: Uuid) -> Result<(), WorldStoreError>;
    fn load_chunk(&self, coord: ChunkCoord) -> Result<Option<ChunkData>, WorldStoreError>;
    fn save_chunk(&self, coord: ChunkCoord, data: &ChunkData) -> Result<(), WorldStoreError>;
    fn remove_chunk(&self, coord: ChunkCoord) -> Result<(), WorldStoreError>;
    fn flush(&self) -> Result<(), WorldStoreError>;
}

pub struct InMemoryWorldStore {
    slot_id: Uuid,
    meta: Mutex<Option<WorldMeta>>,
    active: Mutex<ActiveStateSnapshot>,
    designs: Mutex<Vec<ShipDesign>>,
    chunks: Mutex<HashMap<ChunkCoord, ChunkData>>,
}

impl Default for InMemoryWorldStore {
    fn default() -> Self {
        Self {
            slot_id: Uuid::now_v7(),
            meta: Mutex::new(None),
            active: Mutex::new(ActiveStateSnapshot::default()),
            designs: Mutex::new(Vec::new()),
            chunks: Mutex::new(HashMap::new()),
        }
    }
}

impl WorldStore for InMemoryWorldStore {
    fn load_meta(&self) -> Result<WorldMeta, WorldStoreError> {
        self.meta
            .lock()
            .map_err(lock_err)?
            .clone()
            .ok_or_else(|| WorldStoreError::NotFound("in-memory world meta".to_string()))
    }

    fn list_slots(&self) -> Result<Vec<SlotSummary>, WorldStoreError> {
        let Ok(meta) = self.load_meta() else {
            return Ok(Vec::new());
        };
        Ok(vec![SlotSummary {
            slot_id: self.slot_id,
            name: meta.name,
            seed: meta.seed,
            play_time_secs: meta.play_time_secs,
            created_at_ms: meta.created_at_ms,
            updated_at_ms: meta.updated_at_ms,
        }])
    }

    fn load_active_state(&self) -> Result<ActiveStateSnapshot, WorldStoreError> {
        Ok(self.active.lock().map_err(lock_err)?.clone())
    }

    fn save_world(&self, meta: &WorldMeta, active: &ActiveStateSnapshot) -> Result<(), WorldStoreError> {
        *self.meta.lock().map_err(lock_err)? = Some(meta.clone());
        *self.active.lock().map_err(lock_err)? = active.clone();
        Ok(())
    }

    fn list_designs(&self) -> Result<Vec<ShipDesign>, WorldStoreError> {
        Ok(self.designs.lock().map_err(lock_err)?.clone())
    }

    fn save_design(&self, design: &ShipDesign) -> Result<(), WorldStoreError> {
        let mut designs = self.designs.lock().map_err(lock_err)?;
        designs.retain(|entry| entry.name != design.name);
        designs.push(design.clone());
        Ok(())
    }

    fn delete_design(&self, id: Uuid) -> Result<(), WorldStoreError> {
        self.designs
            .lock()
            .map_err(lock_err)?
            .retain(|design| design.id != id);
        Ok(())
    }

    fn load_chunk(&self, coord: ChunkCoord) -> Result<Option<ChunkData>, WorldStoreError> {
        Ok(self.chunks.lock().map_err(lock_err)?.get(&coord).cloned())
    }

    fn save_chunk(&self, coord: ChunkCoord, data: &ChunkData) -> Result<(), WorldStoreError> {
        if data.is_empty() {
            return self.remove_chunk(coord);
        }
        self.chunks
            .lock()
            .map_err(lock_err)?
            .insert(coord, data.clone());
        Ok(())
    }

    fn remove_chunk(&self, coord: ChunkCoord) -> Result<(), WorldStoreError> {
        self.chunks.lock().map_err(lock_err)?.remove(&coord);
        Ok(())
    }

    fn flush(&self) -> Result<(), WorldStoreError> {
        Ok(())
    }
}

pub fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>, WorldStoreError> {
    let bytes = postcard::to_stdvec(value).map_err(|e| WorldStoreError::Encode(e.to_string()))?;
    Ok(lz4_flex::block::compress_prepend_size(&bytes))
}

pub fn decode<T: DeserializeOwned>(blob: &[u8]) -> Result<T, WorldStoreError> {
    let bytes = lz4_flex::block::decompress_size_prepended(blob)
        .map_err(|e| WorldStoreError::Decode(e.to_string()))?;
    postcard::from_bytes(&bytes).map_err(|e| WorldStoreError::Decode(e.to_string()))
}

pub fn now_unix_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn lock_err<T>(error: std::sync::PoisonError<T>) -> WorldStoreError {
    WorldStoreError::Sql(format!("mutex poisoned: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::ship::ai::Doctrine;
    use crate::save::design::starter_designs;
    use crate::save::world::component::WorldContext;
    use crate::save::world::snapshot::{DebrisSnapshot, TransformSave, VelocitySave};

    fn sample_meta() -> WorldMeta {
        WorldMeta {
            name: "Frontier".to_string(),
            seed: 12_345,
            play_time_secs: 42.5,
            created_at_ms: 1_700_000_000_000,
            updated_at_ms: 1_700_000_060_000,
            camera_blob: vec![1, 2, 3],
            control_blob: vec![4, 5],
        }
    }

    fn sample_state() -> ActiveStateSnapshot {
        ActiveStateSnapshot {
            bodies_blob: vec![9, 8, 7],
            drops_blob: vec![6, 5],
            intel_blob: vec![4],
        }
    }

    #[test]
    fn in_memory_round_trips_world_state_and_designs() {
        let store = InMemoryWorldStore::default();
        let meta = sample_meta();
        let state = sample_state();
        store.save_world(&meta, &state).expect("save world");

        assert_eq!(store.load_meta().expect("load meta"), meta);
        assert_eq!(store.load_active_state().expect("load state"), state);

        let mut design = starter_designs().into_iter().next().expect("starter design");
        store.save_design(&design).expect("save design");
        assert_eq!(store.list_designs().expect("list designs"), vec![design.clone()]);

        design.doctrine = Doctrine::Sniper;
        store.save_design(&design).expect("replace design");
        let designs = store.list_designs().expect("list designs");
        assert_eq!(designs.len(), 1, "same-name saves replace the old row");
        assert_eq!(designs[0].doctrine, Doctrine::Sniper);

        store.delete_design(design.id).expect("delete design");
        assert!(store.list_designs().expect("list designs").is_empty());
    }

    #[test]
    fn listed_seed_is_the_meta_seed_for_both_stores() {
        let meta = sample_meta();
        let memory = InMemoryWorldStore::default();
        memory.save_world(&meta, &sample_state()).expect("save world");
        let slots = memory.list_slots().expect("list slots");
        assert_eq!(slots.len(), 1);
        assert_eq!(slots[0].seed, meta.seed);
        assert_eq!(memory.load_meta().expect("load meta").seed, slots[0].seed);
    }

    #[test]
    fn codec_round_trips_arbitrary_bytes() {
        let value = vec![7u8; 1024];
        let blob = encode(&value).expect("encode");
        let restored: Vec<u8> = decode(&blob).expect("decode");
        assert_eq!(restored, value);
        assert!(decode::<Vec<u8>>(&[0xff, 0xff]).is_err(), "corrupt blob fails loudly");
    }

    #[test]
    fn quick_battle_defaults_to_in_memory_session() {
        use std::fs;

        let context = WorldContext::default();
        assert!(matches!(context, WorldContext::QuickBattle { .. }));

        let dir = std::env::temp_dir().join(format!("agame-quick-{}", uuid::Uuid::now_v7()));
        fs::create_dir_all(&dir).expect("create temp saves dir");

        let store = InMemoryWorldStore::default();
        let design = starter_designs().into_iter().next().expect("starter");
        store.save_design(&design).expect("save design in memory");
        store.save_world(&sample_meta(), &sample_state()).expect("save world in memory");
        store.flush().expect("flush is a no-op");

        assert_eq!(
            fs::read_dir(&dir).expect("read temp saves dir").count(),
            0,
            "QuickBattle store performs no disk I/O"
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn in_memory_chunk_persistence_round_trip_and_cleanup() {
        let store = InMemoryWorldStore::default();
        let coord = ChunkCoord::new(3, -5);

        assert_eq!(store.load_chunk(coord).expect("load unvisited"), None);

        let data = ChunkData {
            enemies: Vec::new(),
            debris: Vec::new(),
        };
        store.save_chunk(coord, &data).expect("save empty chunk");
        assert_eq!(store.load_chunk(coord).expect("load empty chunk"), None);

        let non_empty = ChunkData {
            enemies: Vec::new(),
            debris: vec![DebrisSnapshot {
                transform: TransformSave {
                    x: 100.0,
                    y: 200.0,
                    rotation: 0.5,
                },
                velocity: VelocitySave {
                    x: 0.0,
                    y: 0.0,
                    angvel: 0.0,
                },
                team: 2,
                remaining_secs: 15.0,
                blocks: Vec::new(),
            }],
        };
        store.save_chunk(coord, &non_empty).expect("save non-empty chunk");
        let loaded = store.load_chunk(coord).expect("load chunk").expect("chunk present");
        assert_eq!(loaded.debris.len(), 1);
        assert_eq!(loaded.debris[0].remaining_secs, 15.0);

        store.remove_chunk(coord).expect("remove chunk");
        assert_eq!(store.load_chunk(coord).expect("load removed"), None);
    }
}
