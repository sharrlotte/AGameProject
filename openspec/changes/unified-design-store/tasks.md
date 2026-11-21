## 1. Define `DesignStore` Trait and Implementations

- [ ] 1.1 Create `core/src/save/design/store.rs` defining `DesignStore` (`list`, `save`, `delete`), `FileDesignStore`, and `SlotDesignStore`.
- [ ] 1.2 Fix starter seeding in `FileDesignStore` (`save/design/lib.rs`) to write each starter `.ron` file only when `!path.exists()`, preserving player edits.
- [ ] 1.3 Standardize canonical key deduplication (`design_key`) across `FileDesignStore`, `SlotDesignStore`, SQLite `save_design`, and `ShipCatalog::upsert`.

## 2. Remove `DesignCatalogSource` and Wire Session Store Swaps

- [ ] 2.1 Remove `DesignCatalogSource` resource and the stale `ShipCatalog.source` field from `core/src/save/design/component.rs` and `save/design/mod.rs`.
- [ ] 2.2 Refactor `persist_design_request` (`core/src/save/design/system.rs`) to call the active `DesignStore::save` unconditionally with zero backend matching.
- [ ] 2.3 Update `core/src/save/world/system.rs` (`init_world_session`, `handle_load_slot`, `handle_create_slot`, `handle_continue_world`, `handle_quick_battle`, `reset_catalog_on_home`) to install `SlotDesignStore` or `FileDesignStore` and refresh `ShipCatalog` via `DesignStore::list`.
- [ ] 2.4 Remove duplicate `add_message::<ShipSavesChanged>()` registration from `core/src/scene/editor/mod.rs`.

## 3. Verification and Docs

- [ ] 3.1 Add unit tests covering `FileDesignStore` (non-clobbering starter seeding, atomic write, delete), `SlotDesignStore` (per-slot isolation), and consistent key deduplication across both stores.
- [ ] 3.2 Update `docs/save.md` and `docs/scenes.md` (spec only, no notes) to document `DesignStore`, `FileDesignStore`, and `SlotDesignStore`.
