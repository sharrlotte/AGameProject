## Context

Two design storage paths currently coexist with subtle behavioral mismatches:
1. `save/design/component.rs:9-27` defines `DesignCatalogSource` as a standalone `Resource` AND duplicates `pub source: DesignCatalogSource` on `ShipCatalog` (initialized once and never updated again).
2. `save/design/system.rs:23-31` matches `*source` to choose between `context.store().save_design(&event.design)` and `catalog.save_sandbox(&event.design)`, while `save/world/system.rs` repeats `*source = ...` in 6 separate handlers.
3. `save/design/lib.rs:71-76` unconditionally executes `fs::write(&path, text)` for all 6 starter ships on every `load_or_seed_designs_in` call (every startup, Home entry, and QuickBattle launch), silently clobbering any player edits to starter designs in Home's Ship Editor.
4. Sandbox RON files deduplicate by sanitized `design_file_name(design)`, whereas SQLite `save_design` (`sqlite.rs:307`) deduplicates by exact `name`, causing inconsistent behavior when names differ only in case or punctuation.

## Goals / Non-Goals

**Goals:**
- Unify sandbox RON design persistence and per-slot SQLite design persistence behind one `DesignStore` trait while keeping data strictly isolated per save slot vs. Home sandbox.
- Remove `DesignCatalogSource` and all branching in `persist_design_request` and UI systems.
- Fix starter design clobbering in `FileDesignStore` (seed only if missing or empty).
- Unify design replacement/deduplication rules and remove duplicate `ShipSavesChanged` registration.

**Non-Goals:**
- Merging sandbox and save-slot designs into a shared store.
- Redesigning Ship Editor's save UX (such as explicit Save As / Delete buttons).

## Decisions

### 1. `DesignStore` Trait (`FileDesignStore` + `SlotDesignStore`)
- **Choice**: Define in `core/src/save/design/store.rs`:
  ```rust
  pub trait DesignStore: Send + Sync {
      fn list(&self, registry: &BlockRegistry) -> Result<Vec<ShipDesign>, String>;
      fn save(&self, design: &ShipDesign) -> Result<(), String>;
      fn delete(&self, id: Uuid) -> Result<(), String>;
  }
  ```
  - `FileDesignStore { dir: PathBuf }`: reads `<dir>/*.ron` via `save/document.rs`, seeds missing starter `.ron` files only when `!path.exists()` (never overwriting user edits), writes atomically via `write_document`, and supports deleting `<dir>/<key>.ron`.
  - `SlotDesignStore`: delegates to the active slot's SQLite `designs` table (or `IoWorker` handle when `async-save-io` is applied).
  - Store `Box<dyn DesignStore>` inside `ShipCatalog` (or an `ActiveDesignStore` resource paired with `ShipCatalog`), so `persist_design_request` calls `store.save(&event.design)` unconditionally with zero `match` branching.
- **Rationale**: Applies Dependency Inversion (`SOLID`): UI and `DesignSavePlugin` depend solely on `DesignStore`, while `WorldSavePlugin` swaps the backing implementation on session transitions (`FileDesignStore` on Home/Skirmish, `SlotDesignStore` on Campaign save slot).
- **Alternative Considered**: *Keep `DesignCatalogSource` enum and clean up its duplicate field* — still leaks storage-backend branching into `DesignSavePlugin` and blocks adding future backends (e.g., workshop/blueprint import or read-only preset store) cleanly.

### 2. Canonical Keying & Non-Clobbering Seeding
- **Choice**:
  - Use `design_key(&design.name)` (sanitized lowercase ASCII alphanumeric + `_`, matching existing `design_file_name`) consistently for `ShipCatalog::upsert`, `FileDesignStore`, and SQLite name replacement so `"Interceptor"` and `"interceptor"` never create duplicates in SQLite while colliding in RON.
  - In `FileDesignStore::seed_missing_starters()`, check `if !path.exists()` before writing each starter `.ron` file so customized sandbox starters survive returning to Home or starting a Skirmish.
  - Remove the duplicate `app.add_message::<ShipSavesChanged>()` from `core/src/scene/editor/mod.rs:102` (owned solely by `DesignSavePlugin`).

## Risks / Trade-offs

- **Sharing store handle with `WorldContext` / `IoWorker`** → If applied before `async-save-io`, `SlotDesignStore` can wrap a lightweight `Arc<dyn WorldStore>` (or `PathBuf`/handle); when combined with `async-save-io`, `SlotDesignStore` dispatches through `IoWorker`'s command channel cleanly. Both designs keep `DesignStore` stable.
- **User wants to reset modified sandbox starter later** → Non-clobbering seeding preserves user work by default; a future "Reset Starters" button can call `seed_starters_force()` explicitly.
