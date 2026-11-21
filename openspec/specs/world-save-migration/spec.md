# World Save Migration

## Purpose
Sequential versioned schema migration ladder for SQLite save slots to support non-destructive upgrades across versions.
## Requirements
### Requirement: Strict schema version cutover without forking
`SqliteWorldStore::open` and `list_slots` SHALL compare `PRAGMA user_version` against `WORLD_DB_SCHEMA_VERSION` with strict equality. Any database whose version is lower or higher SHALL be rejected with `WorldStoreError::VersionMismatch { expected, found }` without modifying, migrating, or copying the `.db` file.

#### Scenario: Lower schema version is rejected untouched
- **WHEN** opening a save slot whose `PRAGMA user_version` is less than `WORLD_DB_SCHEMA_VERSION`
- **THEN** `SqliteWorldStore::open` returns `WorldStoreError::VersionMismatch`, leaves the original file untouched, and creates no forked `.db` file

#### Scenario: Higher schema version is rejected untouched
- **WHEN** opening a save slot whose `PRAGMA user_version` is greater than `WORLD_DB_SCHEMA_VERSION`
- **THEN** `SqliteWorldStore::open` returns `WorldStoreError::VersionMismatch` without modifying any files

