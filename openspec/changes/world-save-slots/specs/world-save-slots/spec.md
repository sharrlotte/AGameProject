## ADDED Requirements

### Requirement: Each save slot is a single self-contained SQLite file
The game SHALL store each save slot as a single SQLite database file at `<data_dir>/saves/<slot_id>.db` using WAL journal mode and `PRAGMA user_version = WORLD_DB_SCHEMA_VERSION`. Each database SHALL contain `world_meta`, `active_state`, `designs`, `quests`, and `chunks` tables, encoding binary payloads with `postcard` and `lz4_flex`.

#### Scenario: Creating a new save slot produces one database file seeded with starter designs
- **WHEN** the player creates a new save slot named "Frontier"
- **THEN** a single `<data_dir>/saves/<uuid>.db` file is created with `world_meta` (name, seed, timestamps), seeded starter `designs`, and valid `PRAGMA user_version`

#### Scenario: Atomic single-transaction world save
- **WHEN** a world save executes during an active save-slot session
- **THEN** `world_meta` and `active_state` commit together in one SQLite transaction so a crash cannot leave partial or mismatched save files

#### Scenario: Schema version mismatch fails loudly
- **WHEN** opening a `.db` file whose `PRAGMA user_version` does not match `WORLD_DB_SCHEMA_VERSION`
- **THEN** loading returns an explicit `WorldStoreError::VersionMismatch` error without modifying the file

### Requirement: WorldStore trait abstracts SQLite and in-memory persistence
All world persistence SHALL go through a `WorldStore` trait implemented by `SqliteWorldStore` (for save slots) and `InMemoryWorldStore` (for `QuickBattle` and unit tests).

#### Scenario: InMemoryWorldStore round-trips world state and designs with zero disk I/O
- **WHEN** unit tests or `QuickBattle` save and reload `WorldMeta`, `ActiveStateSnapshot`, and `ShipDesign` via `InMemoryWorldStore`
- **THEN** all data round-trips accurately without creating or modifying any files on disk

### Requirement: Save Slot Picker on Home screen
Clicking `Play` on the `Home` screen SHALL open a Save Slot Picker listing all valid slots under `<data_dir>/saves/*.db` sorted by most recently updated, reading only `world_meta` for each slot. The picker SHALL support `New World`, `Load` (continue selected slot), and `Delete` (with 2-step confirmation removing `.db`, `.db-wal`, and `.db-shm`).

#### Scenario: Slot listing reads metadata only
- **WHEN** the Save Slot Picker opens with multiple `.db` files on disk
- **THEN** each slot displays its name, play time, and last-updated timestamp by querying only `world_meta`

#### Scenario: Deleting a save slot removes database and sidecar files
- **WHEN** the player confirms deleting a save slot in the Save Slot Picker
- **THEN** its `.db`, `.db-wal`, and `.db-shm` files are removed and the slot disappears from the list

### Requirement: Per-slot ship designs with Home sandbox isolation
When inside an active save slot, `ShipCatalog` and the Ship Editor SHALL read and write that save slot's `designs` table (`DesignCatalogSource::ActiveWorldDb`). When launched from `Home`, the Ship Editor and `QuickBattle` SHALL use `DesignCatalogSource::HomeSandbox` without touching any save slot `.db`.

#### Scenario: Saving a design inside a save slot isolates it to that slot
- **WHEN** the player opens the Ship Editor from Pause inside Slot A, saves a modified ship design, and later opens Slot B
- **THEN** Slot A contains the modified design and Slot B's `designs` table is unaffected

#### Scenario: Home Ship Editor and QuickBattle do not touch save slots
- **WHEN** the player edits a ship from Home or plays `QuickBattle`
- **THEN** no file in `<data_dir>/saves/*.db` is created, modified, or deleted
