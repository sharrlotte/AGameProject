## 1. Storage & Migration Atomicity

- [x] 1.1 Move `now_unix_ms` to `core/src/save/world/store.rs` and deduplicate imports in `sqlite.rs` and `system.rs`
- [x] 1.2 Refactor `save_design` in `SqliteWorldStore` to execute stale deletion (`DELETE FROM designs WHERE name = ?1 AND id <> ?2`) and design insertion within a single `IMMEDIATE` transaction
- [x] 1.3 Implement sequential schema migration ladder in `SqliteWorldStore` (`migrate()` method running `v0 -> v1`, updating `user_version`, and rejecting downgrades with `VersionMismatch`)

## 2. Event Dispatch & Verified Persistence

- [x] 2.1 Move `ShipSavesChanged` definition to `core/src/save/design/component.rs` and re-export in `save/design/mod.rs`
- [x] 2.2 Update `persist_design_request` in `core/src/save/design/system.rs` to write `ShipSavesChanged` only after successful save
- [x] 2.3 Remove immediate `ship_saved_event.write(ShipSavesChanged)` from `core/src/scene/editor/save.rs` and update `core/src/scene/editor/mod.rs` to import from `save::design`

## 3. World Snapshot Modularity

- [x] 3.1 Create `core/src/save/world/snapshot/component.rs` containing all snapshot serde data structures
- [x] 3.2 Consolidate ship conversions and capture/apply logic into `capture.rs` and `apply.rs`
- [x] 3.3 Consolidate debris capture and restore into `capture.rs` and `apply.rs`
- [x] 3.4 Create `core/src/save/world/snapshot/capture.rs` containing pure ECS capture functions
- [x] 3.5 Create `core/src/save/world/snapshot/apply.rs` containing ECS restoration functions
- [x] 3.6 Create `core/src/save/world/snapshot/mod.rs` defining `BodySnapshot` and re-exporting all snapshot symbols, then remove `core/src/save/world/snapshot.rs`

## 4. System Decomposition & Verification

- [x] 4.1 Decompose `spawn_loaded_world` in `core/src/save/world/system.rs` into `load_world_data` helper (I/O & decode) and `apply_world_data` helper (ECS restoration)
- [x] 4.2 Add unit tests for atomic design upsert and sequential schema migration in `core/src/save/world/sqlite.rs`
- [x] 4.3 Update `/docs/save.md` and `/docs/architecture.md` to reflect the refined module structure and migration architecture
- [x] 4.4 Run unit tests to verify round-trips and migration ladder pass cleanly
