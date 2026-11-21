## MODIFIED Requirements

### Requirement: Each save slot is a single self-contained SQLite file
The game SHALL store each save slot as a single SQLite database file at `<data_dir>/saves/<slot_id>.db` using WAL journal mode and `PRAGMA user_version = WORLD_DB_SCHEMA_VERSION`. Each database SHALL contain `world_meta`, `active_state`, `designs`, `quests`, and `chunks` (with `(x, y)` primary key, `visited`, and per-subsystem nullable `BLOB` columns starting with `bodies_blob`) tables, encoding binary payloads with `postcard` and `lz4_flex`. `world_meta` SHALL store the authoritative seed in a `seed` column and SHALL NOT store a separate `rng_blob`.

#### Scenario: Creating a new save slot produces one database file seeded with starter designs
- **WHEN** the player creates a new save slot named "Frontier"
- **THEN** a single `<data_dir>/saves/<uuid>.db` file is created with `world_meta`, seeded starter `designs`, multi-column `chunks`, and valid `PRAGMA user_version`

#### Scenario: Atomic single-transaction world save includes dirty cached chunks
- **WHEN** a world save executes during an active save-slot session while `ChunkCache` holds dirty chunks
- **THEN** `world_meta`, `active_state`, `quests`, and all dirty `chunks` rows commit together in one `BEGIN IMMEDIATE` SQLite transaction so a crash cannot leave active entities and chunk rows inconsistent

#### Scenario: Schema version mismatch fails loudly
- **WHEN** opening a `.db` file whose `PRAGMA user_version` differs from `WORLD_DB_SCHEMA_VERSION` in either direction
- **THEN** loading returns an explicit `WorldStoreError::VersionMismatch` error without modifying the file
