## MODIFIED Requirements

### Requirement: Each save slot is a single self-contained SQLite file
The game SHALL store each save slot as a single SQLite database file at `<data_dir>/saves/<slot_id>.db` using WAL journal mode and `PRAGMA user_version = WORLD_DB_SCHEMA_VERSION`. Each database SHALL contain `world_meta`, `active_state`, `designs`, `quests`, and `chunks` tables, encoding binary payloads with `postcard` and `lz4_flex`.

#### Scenario: Creating a new save slot produces one database file seeded with starter designs
- **WHEN** the player creates a new save slot named "Frontier"
- **THEN** a single `<data_dir>/saves/<uuid>.db` file is created with `world_meta` (name, seed, timestamps), seeded starter `designs`, and valid `PRAGMA user_version`

#### Scenario: Atomic single-transaction world save
- **WHEN** a world save executes during an active save-slot session
- **THEN** `world_meta` and `active_state` commit together in one SQLite transaction so a crash cannot leave partial or mismatched save files

#### Scenario: Schema version mismatch fails loudly
- **WHEN** opening a `.db` file whose `PRAGMA user_version` is greater than `WORLD_DB_SCHEMA_VERSION`
- **THEN** loading returns an explicit `WorldStoreError::VersionMismatch` error without modifying the file

### Requirement: Per-slot ship designs with Home sandbox isolation
When inside an active save slot, `ShipCatalog` and the Ship Editor SHALL read and write that save slot's `designs` table (`DesignCatalogSource::ActiveWorldDb`). When launched from `Home`, the Ship Editor and `QuickBattle` SHALL use `DesignCatalogSource::HomeSandbox` without touching any save slot `.db`. All design writes to a save slot SHALL commit atomically in a single SQLite transaction, and `ShipSavesChanged` SHALL only be emitted after persistence succeeds.

#### Scenario: Saving a design inside a save slot isolates it to that slot
- **WHEN** the player opens the Ship Editor from Pause inside Slot A, saves a modified ship design, and later opens Slot B
- **THEN** Slot A contains the modified design, Slot B's `designs` table is unaffected, and design changes commit atomically in a single transaction

#### Scenario: Home Ship Editor and QuickBattle do not touch save slots
- **WHEN** the player edits a ship from Home or plays `QuickBattle`
- **THEN** no file in `<data_dir>/saves/*.db` is created, modified, or deleted

#### Scenario: ShipSavesChanged is only emitted after persistence completes
- **WHEN** a design save is requested
- **THEN** `ShipSavesChanged` is emitted only after the write successfully commits to the underlying store or sandbox directory
