## 1. Dependencies & WorldStore Core

- [ ] 1.1 Add `rusqlite` (with `bundled` feature), `postcard`, and `lz4_flex` to `core/Cargo.toml`
- [ ] 1.2 Create `core/src/save/world/component.rs` defining `WORLD_DB_SCHEMA_VERSION`, `WorldSession`, `SlotSummary`, `WorldMeta`, `ActiveStateSnapshot`, `WorldStoreError`, and `EditorReturnTarget`
- [ ] 1.3 Create `core/src/save/world/store.rs` defining the `WorldStore` trait, `postcard` + `lz4_flex` encode/decode helpers, and `InMemoryWorldStore`
- [ ] 1.4 Create `core/src/save/world/sqlite.rs` implementing `SqliteWorldStore` (WAL mode, `PRAGMA user_version` check, `world_meta`, `active_state`, `designs`, `quests`, `chunks` tables, single-transaction `save_world`, metadata-only `list_slots`, and `delete_slot_files` cleaning `.db`/`.db-wal`/`.db-shm`)

## 2. Snapshot Capture/Apply & Per-Slot Designs

- [ ] 2.1 Create `core/src/save/world/snapshot.rs` porting reusable single-ship, single-debris, drop, camera, control, RNG, and `TeamIntel` capture/apply helpers from `save/battle`
- [ ] 2.2 Update `core/src/save/design/` to support `DesignCatalogSource::{ActiveWorldDb, HomeSandbox}` so Ship Editor reads/writes the active slot's `designs` table when opened from gameplay and uses the sandbox catalog when opened from Home
- [ ] 2.3 Create `core/src/save/world/system.rs` and `core/src/save/world/mod.rs` (`WorldSavePlugin`) wiring periodic auto-save, save-on-Pause-exit, slot load/create lifecycle, and stateless `InMemoryWorldStore` for `QuickBattle`
- [ ] 2.4 Remove legacy `core/src/save/battle/`, `core/src/save/campaign/`, and `core/src/save/fleet/` modules and update `SavePlugin`

## 3. Home Save Slot Picker, Pause & Settings UI

- [ ] 3.1 Update `core/src/scene/home/` so `Play` opens the Save Slot Picker (`Load`, `New World`, 2-step `Delete`, displaying name, play time, and last-saved timestamp) while keeping `QuickBattle`, `Ship Editor`, `Settings`, and `Exit`
- [ ] 3.2 Update `core/src/scene/pause/` to add `Ship Editor` (with `EditorReturnTarget::Game`) during `WorldSession::SaveSlot` and flush/save `ActiveWorldStore` on `Back to Home`
- [ ] 3.3 Remove the legacy `Data / Reset Mission / Campaign` section from `core/src/scene/setting/`

## 4. Tests, Documentation & Verification

- [ ] 4.1 Add unit tests covering `SqliteWorldStore` & `InMemoryWorldStore` round-trips (`WorldMeta`, `ActiveStateSnapshot`, per-slot `designs`), `list_slots` metadata-only queries, `delete_slot_files`, `PRAGMA user_version` mismatch rejection, and `QuickBattle` disk isolation
- [ ] 4.2 Update `/docs/save.md`, `/docs/ui.md`, and `/docs/architecture.md`
- [ ] 4.3 Run targeted unit tests (`cargo test`) and verify in-game with `cargo dev`
