## Context

The `world-save-slots` change established single-file SQLite persistence for playthroughs (`<data_dir>/saves/<slot_id>.db`) and a Home-screen save slot picker. A detailed code review identified key opportunities for refinement:
1. `save_design` in `SqliteWorldStore` operated across multiple mutex acquisitions and performed deletes outside the insertion transaction.
2. `ShipSavesChanged` was triggered optimistically before persistence was confirmed by `persist_design_request`.
3. Save slot databases checked schema version via strict equality, with no migration pathway to upgrade save slots across future versions.
4. `spawn_loaded_world` in `system.rs` mixed store I/O, binary decoding, transient despawning, and ECS component mutation in a single 88-line block.
5. `snapshot.rs` (860 lines) significantly exceeded the 300-line modularity rule outlined in project guidelines.
6. `now_unix_ms()` utility was duplicated across `sqlite.rs` and `system.rs`.

## Goals / Non-Goals

**Goals:**
- Guarantee atomic design updates in `SqliteWorldStore` within a single `IMMEDIATE` transaction.
- Establish verified save notification by moving `ShipSavesChanged` dispatch to `persist_design_request`.
- Implement a sequential migration ladder in `SqliteWorldStore` starting from v0 (empty DB) to v1 (`WORLD_DB_SCHEMA_VERSION`), rejecting downgrades while enabling incremental upgrades.
- Decompose `spawn_loaded_world` into staged helpers: pure store I/O + decode (`load_world_data`) and ECS application (`apply_world_data`).
- Refactor `snapshot.rs` into a cohesive `snapshot/` module under 300 lines per file.
- Deduplicate `now_unix_ms` into `core/src/save/world/store.rs`.

**Non-Goals:**
- Introducing asynchronous background I/O threads for SQLite saves (synchronous transaction with WAL mode remains sufficient).
- Changing the binary wire format (`postcard` + `lz4_flex`) or existing table structures for v1.
- Splitting `slots.rs` or `sqlite.rs` in this change (their core implementation lines excluding tests remain cohesive).

## Decisions

### Decision 1: Atomic Design Upsert via Single SQLite Transaction
- **Rationale**: Currently, `save_design` inspects conflicting names, unlocks, calls `delete_design`, unlocks, then opens a transaction for `insert_design`. A crash between delete and insert loses designs.
- **Approach**: Acquire the connection lock once, start an `IMMEDIATE` transaction, run `DELETE FROM designs WHERE name = ?1 AND id <> ?2`, execute `insert_design`, and commit.
- **Alternatives Considered**: Retaining separate delete helper: rejected due to race conditions and lack of atomicity.

### Decision 2: Sequential Schema Migration Ladder
- **Rationale**: Fixed version checking via `user_version == WORLD_DB_SCHEMA_VERSION` bricks existing user saves when new tables or columns are introduced.
- **Approach**: Read `user_version` on `open()`. If `found > WORLD_DB_SCHEMA_VERSION`, fail immediately with `VersionMismatch` (reject downgrade). If `found < WORLD_DB_SCHEMA_VERSION`, step through each version sequentially in a loop (`v0 -> v1`, etc.) updating `user_version` upon completion.
- **Alternatives Considered**: Ad-hoc `ALTER TABLE` try-catch blocks: rejected as brittle and non-deterministic compared to linear version migrations.

### Decision 3: Verified Design Save Notification
- **Rationale**: `ShipSavesChanged` signals the editor UI to refresh catalog views. Firing it in `save.rs` concurrently with sending `DesignSaveRequested` is premature if the disk or database write fails.
- **Approach**: Define `ShipSavesChanged` in `crate::save::design` (or preserve its definition in `scene::editor` without cross-coupling). In `persist_design_request`, emit `ShipSavesChanged` solely after `catalog.save_sandbox()` or `active.save_design()` returns `Ok(())`.

### Decision 4: Decompose `spawn_loaded_world` into Pure and Impure Phases
- **Rationale**: `spawn_loaded_world` mixes error handling, disk reads, decompression, entity despawns, and ECS spawns.
- **Approach**:
  - `load_world_data(active: &dyn WorldStore) -> Result<WorldLoadData, WorldStoreError>`: Reads meta and active snapshots and decodes camera, control, bodies, drops, intel, and RNG into a plain data struct.
  - `apply_world_data(commands, data, registry, config, camera, items)`: Despawns transients and restores entities into Bevy ECS.
- **Alternatives Considered**: Introducing a full `LoadPhase` state machine: rejected as unnecessarily complex for a single-frame transition.

### Decision 5: Modularize `snapshot/` Subfolder
- **Rationale**: `snapshot.rs` is 860 lines, violating the project 300-line rule.
- **Approach**: Split into:
  - `snapshot/component.rs`: Plain data structs and serde types (`BlockSnapshot`, `TransformSave`, etc.).
  - `snapshot/ship.rs`: `ShipSnapshot`, `BrainSave`, `TacticSave` conversions.
  - `snapshot/debris.rs`: `DebrisSnapshot` capture and restore.
  - `snapshot/capture.rs`: Pure ECS read functions (`capture_camera`, `capture_control`, `capture_bodies`, `capture_drops`, `capture_intel`).
  - `snapshot/apply.rs`: ECS write functions (`apply_bodies`, `apply_intel`, `apply_control`, `apply_camera`, `apply_drops`).
  - `snapshot/mod.rs`: `BodySnapshot` enum, re-exports for seamless compatibility.

## Risks / Trade-offs

- **[Risk] Module split breaks external imports** → *Mitigation*: `snapshot/mod.rs` re-exports all structs and functions, preserving `crate::save::world::snapshot::*` imports.
- **[Risk] Migration loop fails mid-way** → *Mitigation*: Each migration step runs inside a transaction and commits before incrementing `user_version`, ensuring failed migrations leave the database at the prior valid version.
