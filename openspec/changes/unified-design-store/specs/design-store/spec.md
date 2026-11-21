## ADDED Requirements

### Requirement: Ship design storage is unified behind a DesignStore trait
All ship design listing, saving, and deletion SHALL execute through a `DesignStore: Send + Sync` trait implemented by `FileDesignStore` (for Home sandbox / Skirmish `.ron` files under `<data_dir>/sandbox_designs/`) and `SlotDesignStore` (for per-slot SQLite `designs` rows). `DesignCatalogSource` SHALL NOT exist, `ShipCatalog` SHALL NOT hold a duplicate `source` enum field, and `persist_design_request` SHALL invoke `DesignStore::save` without backend branching.

#### Scenario: Saving a design uses DesignStore without backend branching
- **WHEN** `DesignSaveRequested` is processed by `persist_design_request`
- **THEN** it persists via the active `DesignStore::save`, updates `ShipCatalog`, and emits `ShipSavesChanged` only after persistence succeeds

#### Scenario: Both stores deduplicate with the same canonical key
- **WHEN** a design named `"My Ship"` is saved and then replaced by `"my_ship"`
- **THEN** both `FileDesignStore` and `SlotDesignStore` replace the existing entry under the same canonical key without creating duplicate entries

### Requirement: Sandbox starter seeding preserves user edits
`FileDesignStore` SHALL seed missing starter design `.ron` files when they do not exist on disk and SHALL NOT overwrite existing starter files on Home entry or Skirmish startup.

#### Scenario: Edited starter survives returning to Home
- **WHEN** a player edits `"Interceptor"` in the Home Ship Editor, saves it, and returns to Home
- **THEN** `<data_dir>/sandbox_designs/interceptor.ron` retains the player's edits instead of being overwritten with the built-in starter text
