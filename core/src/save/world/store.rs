use std::sync::Mutex;

use serde::Serialize;
use serde::de::DeserializeOwned;
use uuid::Uuid;

use crate::content::ship::ShipDesign;
use crate::save::world::component::{ActiveStateSnapshot, WorldMeta, WorldStoreError};

pub trait WorldStore: Send + Sync {
    fn load_meta(&self) -> Result<WorldMeta, WorldStoreError>;
    fn load_active_state(&self) -> Result<ActiveStateSnapshot, WorldStoreError>;
    fn save_world(&self, meta: &WorldMeta, active: &ActiveStateSnapshot) -> Result<(), WorldStoreError>;
    fn list_designs(&self) -> Result<Vec<ShipDesign>, WorldStoreError>;
    fn save_design(&self, design: &ShipDesign) -> Result<(), WorldStoreError>;
    fn delete_design(&self, id: Uuid) -> Result<(), WorldStoreError>;
    fn flush(&self) -> Result<(), WorldStoreError>;
}

#[derive(Default)]
pub struct InMemoryWorldStore {
    meta: Mutex<Option<WorldMeta>>,
    active: Mutex<ActiveStateSnapshot>,
    designs: Mutex<Vec<ShipDesign>>,
}

impl WorldStore for InMemoryWorldStore {
    fn load_meta(&self) -> Result<WorldMeta, WorldStoreError> {
        self.meta
            .lock()
            .map_err(lock_err)?
            .clone()
            .ok_or_else(|| WorldStoreError::NotFound("in-memory world meta".to_string()))
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
    use crate::save::world::component::WorldSession;

    fn sample_meta() -> WorldMeta {
        WorldMeta {
            name: "Frontier".to_string(),
            seed: 12_345,
            play_time_secs: 42.5,
            created_at_ms: 1_700_000_000_000,
            updated_at_ms: 1_700_000_060_000,
            camera_blob: vec![1, 2, 3],
            control_blob: vec![4, 5],
            rng_blob: vec![6],
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

        let session = WorldSession::default();
        assert_eq!(session, WorldSession::QuickBattle);

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
}
