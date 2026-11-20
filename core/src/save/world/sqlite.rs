use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use bevy::prelude::*;
use rusqlite::{Connection, OpenFlags, OptionalExtension, Transaction, TransactionBehavior, params};
use uuid::Uuid;

use crate::content::ship::ShipDesign;
use crate::save::document::data_dir;
use crate::save::world::component::{
    ActiveStateSnapshot, SlotSummary, WorldMeta, WorldStoreError, WORLD_DB_SCHEMA_VERSION,
};
use crate::save::world::store::{WorldStore, decode, encode, now_unix_ms};

pub struct SqliteWorldStore {
    conn: Mutex<Connection>,
}

impl std::fmt::Debug for SqliteWorldStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SqliteWorldStore").finish_non_exhaustive()
    }
}

impl SqliteWorldStore {
    pub fn create(path: &Path, meta: &WorldMeta, designs: &[ShipDesign]) -> Result<Self, WorldStoreError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        if path.exists() {
            return Err(WorldStoreError::InvalidData(format!(
                "save slot already exists: {}",
                path.display()
            )));
        }

        let store = Self {
            conn: Mutex::new(Connection::open(path).map_err(sql_err)?),
        };

        store.configure()?;
        store.migrate()?;
        store.insert_initial(meta, designs)?;
        Ok(store)
    }

    pub fn open(path: &Path) -> Result<Self, WorldStoreError> {
        if !path.is_file() {
            return Err(WorldStoreError::NotFound(path.display().to_string()));
        }

        let store = Self {
            conn: Mutex::new(
                Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_WRITE)
                    .map_err(sql_err)?,
            ),
        };
        store.configure()?;
        store.migrate()?;
        Ok(store)
    }

    fn lock(&self) -> Result<MutexGuard<'_, Connection>, WorldStoreError> {
        self.conn
            .lock()
            .map_err(|e| WorldStoreError::Sql(format!("mutex poisoned: {e}")))
    }

    fn configure(&self) -> Result<(), WorldStoreError> {
        self.lock()?
            .execute_batch("PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;")
            .map_err(sql_err)
    }

    fn migrate(&self) -> Result<(), WorldStoreError> {
        let conn = self.lock()?;
        let found: u32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .map_err(sql_err)?;

        if found > WORLD_DB_SCHEMA_VERSION {
            return Err(WorldStoreError::VersionMismatch {
                expected: WORLD_DB_SCHEMA_VERSION,
                found,
            });
        }

        for version in found..WORLD_DB_SCHEMA_VERSION {
            match version {
                0 => run_v0_to_v1(&conn)?,
                _ => {
                    return Err(WorldStoreError::VersionMismatch {
                        expected: WORLD_DB_SCHEMA_VERSION,
                        found: version,
                    });
                }
            }
            conn.pragma_update(None, "user_version", version + 1)
                .map_err(sql_err)?;
        }

        Ok(())
    }

    fn insert_initial(&self, meta: &WorldMeta, designs: &[ShipDesign]) -> Result<(), WorldStoreError> {
        let mut conn = self.lock()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(sql_err)?;
        {
            tx.execute(
                "INSERT INTO world_meta (id, name, seed, play_time_secs, created_at_ms, updated_at_ms, camera_blob, control_blob, rng_blob)
                 VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    meta.name,
                    meta.seed as i64,
                    meta.play_time_secs,
                    meta.created_at_ms as i64,
                    meta.updated_at_ms as i64,
                    meta.camera_blob,
                    meta.control_blob,
                    meta.rng_blob,
                ],
            )
            .map_err(sql_err)?;
            let empty = ActiveStateSnapshot::default();

            tx.execute(
                "INSERT INTO active_state (id, bodies_blob, drops_blob, intel_blob) VALUES (1, ?1, ?2, ?3)",
                params![empty.bodies_blob, empty.drops_blob, empty.intel_blob],
            )
            .map_err(sql_err)?;

            for design in designs {
                insert_design(&tx, design)?;
            }
        }
        tx.commit().map_err(sql_err)
    }
}

impl WorldStore for SqliteWorldStore {
    fn load_meta(&self) -> Result<WorldMeta, WorldStoreError> {
        self.lock()?
            .query_row(
                "SELECT name, seed, play_time_secs, created_at_ms, updated_at_ms, camera_blob, control_blob, rng_blob FROM world_meta WHERE id = 1",
                [],
                |row| {
                    Ok(WorldMeta {
                        name: row.get(0)?,
                        seed: row.get::<_, i64>(1)? as u64,
                        play_time_secs: row.get(2)?,
                        created_at_ms: row.get::<_, i64>(3)? as u64,
                        updated_at_ms: row.get::<_, i64>(4)? as u64,
                        camera_blob: row.get(5)?,
                        control_blob: row.get(6)?,
                        rng_blob: row.get(7)?,
                    })
                },
            )
            .optional()
            .map_err(sql_err)?
            .ok_or_else(|| WorldStoreError::InvalidData("world_meta row missing".to_string()))
    }

    fn load_active_state(&self) -> Result<ActiveStateSnapshot, WorldStoreError> {
        self.lock()?
            .query_row(
                "SELECT bodies_blob, drops_blob, intel_blob FROM active_state WHERE id = 1",
                [],
                |row| {
                    Ok(ActiveStateSnapshot {
                        bodies_blob: row.get(0)?,
                        drops_blob: row.get(1)?,
                        intel_blob: row.get(2)?,
                    })
                },
            )
            .optional()
            .map_err(sql_err)?
            .ok_or_else(|| WorldStoreError::InvalidData("active_state row missing".to_string()))
    }

    fn save_world(
        &self,
        meta: &WorldMeta,
        active: &ActiveStateSnapshot,
    ) -> Result<(), WorldStoreError> {
        let mut conn = self.lock()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(sql_err)?;
        {
            let updated = tx
                .execute(
                    "UPDATE world_meta SET name = ?1, seed = ?2, play_time_secs = ?3, updated_at_ms = ?4, camera_blob = ?5, control_blob = ?6, rng_blob = ?7 WHERE id = 1",
                    params![
                        meta.name,
                        meta.seed as i64,
                        meta.play_time_secs,
                        meta.updated_at_ms as i64,
                        meta.camera_blob,
                        meta.control_blob,
                        meta.rng_blob,
                    ],
                )
                .map_err(sql_err)?;

            if updated != 1 {
                return Err(WorldStoreError::InvalidData(
                    "world_meta row missing during save".to_string(),
                ));
            }

            tx.execute("DELETE FROM active_state WHERE id = 1", [])
                .map_err(sql_err)?;

            tx.execute(
                "INSERT INTO active_state (id, bodies_blob, drops_blob, intel_blob) VALUES (1, ?1, ?2, ?3)",
                params![active.bodies_blob, active.drops_blob, active.intel_blob],
            )
            .map_err(sql_err)?;
        }

        tx.commit().map_err(sql_err)
    }

    fn list_designs(&self) -> Result<Vec<ShipDesign>, WorldStoreError> {
        let conn = self.lock()?;
        let mut stmt = conn
            .prepare("SELECT data_blob FROM designs ORDER BY name")
            .map_err(sql_err)?;
            
        let rows = stmt
            .query_map([], |row| row.get::<_, Vec<u8>>(0))
            .map_err(sql_err)?;
        
        let mut designs = Vec::new();
        
        for row in rows {
            let blob = row.map_err(sql_err)?;
            designs.push(decode::<ShipDesign>(&blob)?);
        }

        Ok(designs)
    }

    fn save_design(&self, design: &ShipDesign) -> Result<(), WorldStoreError> {
        let mut conn = self.lock()?;
        let tx = conn
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(sql_err)?;

        tx.execute(
            "DELETE FROM designs WHERE name = ?1 AND id <> ?2",
            params![design.name, design.id.to_string()],
        )
        .map_err(sql_err)?;

        insert_design(&tx, design)?;

        tx.commit().map_err(sql_err)
    }

    fn delete_design(&self, id: Uuid) -> Result<(), WorldStoreError> {
        self.lock()?
            .execute("DELETE FROM designs WHERE id = ?1", params![id.to_string()])
            .map_err(sql_err)?;
        Ok(())
    }

    fn flush(&self) -> Result<(), WorldStoreError> {
        self.lock()?
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
            .map_err(sql_err)
    }
}

impl Drop for SqliteWorldStore {
    fn drop(&mut self) {
        if let Err(e) = self.flush() {
            warn!("Failed to checkpoint save slot on close: {e}");
        }
    }
}

pub fn saves_dir() -> PathBuf {
    data_dir().join("saves")
}

pub fn slot_path(slot_id: Uuid) -> PathBuf {
    saves_dir().join(format!("{slot_id}.db"))
}

pub fn list_slots(saves_dir: &Path) -> Vec<SlotSummary> {
    let Ok(entries) = fs::read_dir(saves_dir) else {
        return Vec::new();
    };

    let mut slots = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("db") {
            continue;
        }

        let Some(slot_id) = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .and_then(|stem| Uuid::parse_str(stem).ok())
        else {
            warn!("Skipping save slot with invalid id: {}", path.display());
            continue;
        };

        match read_slot_summary(&path, slot_id) {
            Ok(summary) => slots.push(summary),
            Err(e) => warn!("Skipping save slot {}: {e}", path.display()),
        }
    }
    slots.sort_by(|a, b| b.updated_at_ms.cmp(&a.updated_at_ms));
    slots
}

pub fn delete_slot_files(path: &Path) -> Result<(), WorldStoreError> {
    for candidate in [path.to_path_buf(), wal_path(path), shm_path(path)] {
        match fs::remove_file(&candidate) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(WorldStoreError::Io(format!("{}: {e}", candidate.display()))),
        }
    }
    Ok(())
}

fn read_slot_summary(path: &Path, slot_id: Uuid) -> Result<SlotSummary, WorldStoreError> {
    let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(sql_err)?;
    let found: u32 = conn
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(sql_err)?;

    if found > WORLD_DB_SCHEMA_VERSION || found == 0 {
        return Err(WorldStoreError::VersionMismatch {
            expected: WORLD_DB_SCHEMA_VERSION,
            found,
        });
    }
    
    conn.query_row(
        "SELECT name, seed, play_time_secs, created_at_ms, updated_at_ms FROM world_meta WHERE id = 1",
        [],
        |row| {
            Ok(SlotSummary {
                slot_id,
                name: row.get(0)?,
                seed: row.get::<_, i64>(1)? as u64,
                play_time_secs: row.get(2)?,
                created_at_ms: row.get::<_, i64>(3)? as u64,
                updated_at_ms: row.get::<_, i64>(4)? as u64,
            })
        },
    )
    .map_err(sql_err)
}

fn insert_design(tx: &Transaction<'_>, design: &ShipDesign) -> Result<(), WorldStoreError> {
    let blob = encode(design)?;
    let now_ms = now_unix_ms();
    tx.execute(
        "INSERT INTO designs (id, name, updated_at_ms, data_blob) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(id) DO UPDATE SET name = excluded.name, updated_at_ms = excluded.updated_at_ms, data_blob = excluded.data_blob",
        params![design.id.to_string(), design.name, now_ms as i64, blob],
    )
    .map_err(sql_err)
    .map(|_| ())
}

fn wal_path(path: &Path) -> PathBuf {
    path.with_extension("db-wal")
}

fn shm_path(path: &Path) -> PathBuf {
    path.with_extension("db-shm")
}

fn run_v0_to_v1(conn: &Connection) -> Result<(), WorldStoreError> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS world_meta (
            id             INTEGER PRIMARY KEY CHECK (id = 1),
            name           TEXT NOT NULL,
            seed           INTEGER NOT NULL,
            play_time_secs REAL NOT NULL,
            created_at_ms  INTEGER NOT NULL,
            updated_at_ms  INTEGER NOT NULL,
            camera_blob    BLOB NOT NULL,
            control_blob   BLOB NOT NULL,
            rng_blob       BLOB NOT NULL
        );
        CREATE TABLE IF NOT EXISTS active_state (
            id            INTEGER PRIMARY KEY CHECK (id = 1),
            bodies_blob   BLOB NOT NULL,
            drops_blob    BLOB NOT NULL,
            intel_blob    BLOB NOT NULL
        );
        CREATE TABLE IF NOT EXISTS designs (
            id            TEXT PRIMARY KEY,
            name          TEXT NOT NULL,
            updated_at_ms INTEGER NOT NULL,
            data_blob     BLOB NOT NULL
        );
        CREATE TABLE IF NOT EXISTS quests (
            quest_id      TEXT PRIMARY KEY,
            status        TEXT NOT NULL,
            progress_blob BLOB NOT NULL
        );
        CREATE TABLE IF NOT EXISTS chunks (
            x             INTEGER NOT NULL,
            y             INTEGER NOT NULL,
            visited       INTEGER NOT NULL,
            enemy_count   INTEGER NOT NULL,
            data          BLOB NOT NULL,
            PRIMARY KEY (x, y)
        );
        ",
    )
    .map_err(sql_err)
}

fn sql_err(error: rusqlite::Error) -> WorldStoreError {
    WorldStoreError::Sql(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::ship::ai::Doctrine;
    use crate::save::design::starter_designs;
    use std::fs;

    fn temp_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("agame-world-{label}-{}", Uuid::now_v7()));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn meta() -> WorldMeta {
        WorldMeta {
            name: "Frontier".to_string(),
            seed: 9_876,
            play_time_secs: 12.25,
            created_at_ms: 1_700_000_000_000,
            updated_at_ms: 1_700_000_010_000,
            camera_blob: encode(&"camera").expect("encode camera"),
            control_blob: encode(&"control").expect("encode control"),
            rng_blob: encode(&"rng").expect("encode rng"),
        }
    }

    fn state() -> ActiveStateSnapshot {
        ActiveStateSnapshot {
            bodies_blob: encode(&vec![1u8, 2, 3]).expect("encode bodies"),
            drops_blob: encode(&vec![4u8, 5]).expect("encode drops"),
            intel_blob: encode(&vec![6u8]).expect("encode intel"),
        }
    }

    #[test]
    fn sqlite_round_trips_meta_state_and_slot_designs() {
        let dir = temp_dir("roundtrip");
        let id = Uuid::now_v7();
        let path = dir.join(format!("{id}.db"));
        let initial_designs = starter_designs();

        let store = SqliteWorldStore::create(&path, &meta(), &initial_designs).expect("create slot");
        assert!(path.is_file(), "one .db file per slot");
        assert_eq!(store.load_meta().expect("load meta"), meta());
        assert_eq!(
            store.load_active_state().expect("load state"),
            ActiveStateSnapshot::default(),
            "new slots start with an empty active state"
        );
        let listed = store.list_designs().expect("list designs");
        assert_eq!(listed.len(), initial_designs.len());

        let mut edited = listed[0].clone();
        let edited_name = edited.name.clone();
        edited.doctrine = Doctrine::Sniper;
        store.save_design(&edited).expect("save design");
        let listed = store.list_designs().expect("list designs");
        assert_eq!(listed.len(), initial_designs.len(), "same-name design replaced, not duplicated");
        assert!(listed
            .iter()
            .any(|d| d.name == edited_name && d.doctrine == Doctrine::Sniper));

        let mut changed_meta = meta();
        changed_meta.name = "Renamed World".to_string();
        changed_meta.updated_at_ms += 1000;
        let changed_state = state();
        store.save_world(&changed_meta, &changed_state).expect("save world");
        store.flush().expect("flush");
        drop(store);

        let reopened = SqliteWorldStore::open(&path).expect("reopen slot");
        assert_eq!(reopened.load_meta().expect("reload meta"), changed_meta);
        assert_eq!(reopened.load_active_state().expect("reload state"), changed_state);
        drop(reopened);

        let slots = list_slots(&dir);
        assert_eq!(slots.len(), 1);
        assert_eq!(slots[0].slot_id, id);
        assert_eq!(slots[0].name, "Renamed World");
        assert_eq!(slots[0].seed, 9_876);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn list_slots_reads_only_metadata_and_sorts_newest_first() {
        let dir = temp_dir("list");
        let older = Uuid::now_v7();
        let newer = Uuid::now_v7();
        let mut old_meta = meta();
        old_meta.name = "Old".to_string();
        old_meta.updated_at_ms = 100;
        let mut new_meta = meta();
        new_meta.name = "New".to_string();
        new_meta.updated_at_ms = 200;

        let old_store = SqliteWorldStore::create(&dir.join(format!("{older}.db")), &old_meta, &[]).expect("old slot");
        let new_store = SqliteWorldStore::create(&dir.join(format!("{newer}.db")), &new_meta, &[]).expect("new slot");
        drop(old_store);
        drop(new_store);

        let slots = list_slots(&dir);
        assert_eq!(slots.len(), 2);
        assert_eq!(slots[0].name, "New", "most recently updated first");
        assert_eq!(slots[1].name, "Old");

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn delete_slot_files_removes_db_and_sidecars() {
        let dir = temp_dir("delete");
        let path = dir.join(format!("{}.db", Uuid::now_v7()));
        for file in [path.clone(), wal_path(&path), shm_path(&path)] {
            fs::write(&file, b"data").expect("create sidecar");
        }

        delete_slot_files(&path).expect("delete all slot files");
        assert!(!path.exists());
        assert!(!wal_path(&path).exists());
        assert!(!shm_path(&path).exists());
        delete_slot_files(&path).expect("delete is idempotent");

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn version_mismatch_is_rejected_without_modifying_file() {
        let dir = temp_dir("version");
        let path = dir.join(format!("{}.db", Uuid::now_v7()));
        let store = SqliteWorldStore::create(&path, &meta(), &[]).expect("create slot");
        drop(store);

        {
            let conn = Connection::open(&path).expect("open raw db");
            conn.pragma_update(None, "user_version", 999u32).expect("set bad version");
        }

        let error = SqliteWorldStore::open(&path).expect_err("incompatible db is rejected");
        assert!(
            matches!(
                error,
                WorldStoreError::VersionMismatch {
                    expected: WORLD_DB_SCHEMA_VERSION,
                    found: 999
                }
            ),
            "{error}"
        );
        assert!(list_slots(&dir).is_empty(), "incompatible slots are skipped");

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn sequential_migration_ladder_upgrades_schema() {
        let dir = temp_dir("migration");
        let path = dir.join(format!("{}.db", Uuid::now_v7()));

        {
            let conn = Connection::open(&path).expect("create empty raw db");
            let v: u32 = conn
                .pragma_query_value(None, "user_version", |row| row.get(0))
                .expect("query version");
            assert_eq!(v, 0, "new raw db starts at user_version 0");
        }

        let store = SqliteWorldStore::open(&path).expect("open runs migration ladder");
        let conn = store.lock().expect("lock connection");
        let v: u32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("query migrated version");
        assert_eq!(v, WORLD_DB_SCHEMA_VERSION);

        let count: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name IN ('world_meta', 'active_state', 'designs', 'quests', 'chunks')",
                [],
                |row| row.get(0),
            )
            .expect("count migrated tables");
        assert_eq!(count, 5, "all 5 schema tables exist after migration");

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn atomic_design_save_replaces_same_name_different_id() {
        let dir = temp_dir("atomic_design");
        let path = dir.join(format!("{}.db", Uuid::now_v7()));
        let store = SqliteWorldStore::create(&path, &meta(), &[]).expect("create slot");

        let mut design_a = starter_designs().into_iter().next().expect("starter");
        design_a.name = "Custom Cruiser".to_string();
        design_a.id = Uuid::now_v7();
        store.save_design(&design_a).expect("save design A");

        let mut design_b = design_a.clone();
        design_b.id = Uuid::now_v7();
        design_b.doctrine = Doctrine::Sniper;
        store.save_design(&design_b).expect("save design B with same name");

        let designs = store.list_designs().expect("list designs");
        assert_eq!(designs.len(), 1, "only 1 design remains with this name");
        assert_eq!(designs[0].id, design_b.id, "id updated to design B");
        assert_eq!(designs[0].doctrine, Doctrine::Sniper);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn opening_missing_slot_creates_no_file() {
        let dir = temp_dir("missing");
        let path = dir.join(format!("{}.db", Uuid::now_v7()));
        assert!(matches!(
            SqliteWorldStore::open(&path),
            Err(WorldStoreError::NotFound(_))
        ));
        assert!(!path.exists(), "open must never create a missing slot");
        let _ = fs::remove_dir_all(dir);
    }
}
