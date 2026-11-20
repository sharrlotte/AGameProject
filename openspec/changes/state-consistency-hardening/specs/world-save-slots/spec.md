## MODIFIED Requirements

### Requirement: Each save slot is a single self-contained SQLite file
The game SHALL store each save slot as a single SQLite database file at `<data_dir>/saves/<slot_id>.db` using WAL journal mode and `PRAGMA user_version = WORLD_DB_SCHEMA_VERSION` (2). Each database SHALL contain `world_meta`, `active_state`, `designs`, `quests`, and `chunks` tables, encoding binary payloads with `postcard` and `lz4_flex`. `world_meta` SHALL store the authoritative seed in a `seed` column and SHALL NOT store a separate `rng_blob`.

#### Scenario: Creating a new save slot produces one database file seeded with starter designs
- **WHEN** the player creates a new save slot named "Frontier"
- **THEN** a single `<data_dir>/saves/<uuid>.db` file is created with `world_meta` (name, seed, timestamps), seeded starter `designs`, and valid `PRAGMA user_version`

#### Scenario: Atomic single-transaction world save
- **WHEN** a world save executes during an active save-slot session
- **THEN** `world_meta` and `active_state` commit together in one SQLite transaction so a crash cannot leave partial or mismatched save files

#### Scenario: Schema version mismatch fails loudly
- **WHEN** opening a `.db` file whose `PRAGMA user_version` differs from `WORLD_DB_SCHEMA_VERSION` in either direction
- **THEN** loading returns an explicit `WorldStoreError::VersionMismatch` error without modifying the file

### Requirement: Save Slot Picker on Home screen
Clicking `Worlds` on the Home screen SHALL open the Save Slot Picker listing all valid slots under `<data_dir>/saves/*.db` sorted by most recently updated, reading only `world_meta` for each slot. Play SHALL continue the last save instead of opening the picker. The picker SHALL support `New World`, `Load` (continue selected slot), and `Delete` (with 2-step confirmation removing `.db`, `.db-wal`, and `.db-shm`).

#### Scenario: Slot listing reads metadata only
- **WHEN** the Save Slot Picker opens with multiple `.db` files on disk
- **THEN** each slot displays its name, play time, and last-updated timestamp by querying only `world_meta`

#### Scenario: Deleting a save slot removes database and sidecar files
- **WHEN** the player confirms deleting a save slot in the Save Slot Picker
- **THEN** its `.db`, `.db-wal`, and `.db-shm` files are removed and the slot disappears from the list

#### Scenario: Play bypasses the picker
- **WHEN** the player clicks Play on Home with a valid last save
- **THEN** that save is restored directly and no picker is shown
