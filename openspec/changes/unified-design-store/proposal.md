## Why

Ship design persistence is split across two incompatible code paths: `DesignCatalogSource::HomeSandbox` writes pretty RON files under `<data_dir>/sandbox_designs/` via `save/design/lib.rs`, while `DesignCatalogSource::ActiveWorldDb` writes `postcard`+`lz4` rows into the active SQLite `designs` table. Because of this split, `persist_design_request` (`save/design/system.rs:23-31`) branches on `DesignCatalogSource`, `ShipCatalog` carries a stale unread duplicate `pub source` field (`save/design/component.rs:26`), `FileDesignStore` deduplicates by sanitized filename whereas SQLite deduplicates by exact `name`, `ShipSavesChanged` is registered twice, and `load_or_seed_designs_in` (`save/design/lib.rs:71-76`) unconditionally overwrites edited starter `.ron` files on every Home/QuickBattle transition.

## What Changes

- Define a `DesignStore: Send + Sync` trait (`list`, `save`, `delete`) with two implementations:
  - `FileDesignStore` for Home sandbox / Skirmish (`<data_dir>/sandbox_designs/*.ron`), seeding starter designs **only when missing** (fixing starter edit clobbering).
  - `SlotDesignStore` backed by the active save slot's SQLite `designs` table (preserving strict per-slot isolation).
- Hold the active `Box<dyn DesignStore>` inside `ActiveDesignStore` (or `ShipCatalog`) and remove the `DesignCatalogSource` enum, the stale `ShipCatalog.source` field, and every source-switching branch from `save/design/system.rs` and `save/world/system.rs`.
- Unify deduplication keying across both stores using one canonical design key (`design_key(name)` / stable `ShipDesign.id`), and remove duplicate `ShipSavesChanged` message registration from `ShipEditorPlugin`.

## Capabilities

### New Capabilities
- `design-store`: Backend-agnostic `DesignStore` trait with `FileDesignStore` (non-clobbering sandbox RON storage) and `SlotDesignStore` (per-slot SQLite storage), eliminating `DesignCatalogSource` branching from catalog and UI systems.

### Modified Capabilities
- `world-save-slots`: Update per-slot ship design isolation requirement to use `DesignStore` (`SlotDesignStore` vs `FileDesignStore`) instead of `DesignCatalogSource::{ActiveWorldDb, HomeSandbox}`.

## Non-Goals

- Merging sandbox designs and per-slot designs into one shared dataset (per-slot isolation remains mandatory).
- Adding new Ship Editor UI features (rename, delete, or new-ship dialog).
- Moving design I/O onto the background worker (handled by `async-save-io`, which `SlotDesignStore`/`FileDesignStore` integrate with cleanly).

## Impact

- `core/src/save/design/` (`mod.rs`, `component.rs`, `lib.rs`, `system.rs`, new `store.rs`).
- `core/src/save/world/system.rs` (swapping active `DesignStore` on slot create/load/continue/Home/QuickBattle instead of mutating `DesignCatalogSource`).
- `core/src/scene/editor/mod.rs` (remove duplicate `add_message::<ShipSavesChanged>()`).
- `docs/save.md` and `docs/scenes.md` (spec-only updates).
