# World Save Migration

## Purpose
Sequential versioned schema migration ladder for SQLite save slots to support non-destructive upgrades across versions.

## Requirements

### Requirement: Sequential save slot schema migration
The game SHALL inspect `PRAGMA user_version` upon opening any existing save slot database file. If `user_version` is less than `WORLD_DB_SCHEMA_VERSION`, it SHALL leave the original file completely untouched, copy it to a new save slot database file at `<data_dir>/saves/<new_uuid>.db`, execute sequential migration steps on the new file up to `WORLD_DB_SCHEMA_VERSION`, and update the world name in the new database to indicate migration. If `user_version` exceeds `WORLD_DB_SCHEMA_VERSION`, it SHALL reject the file with `WorldStoreError::VersionMismatch` without copying or modifying anything.

#### Scenario: Opening an earlier version forks into a new save slot
- **WHEN** opening a save slot database whose `user_version` is lower than `WORLD_DB_SCHEMA_VERSION`
- **THEN** the original database file remains unchanged, a new `<new_uuid>.db` file is created and migrated to `WORLD_DB_SCHEMA_VERSION`, and the active session loads the new file

#### Scenario: Opening a newer database version is rejected
- **WHEN** opening a save slot database whose `user_version` is greater than `WORLD_DB_SCHEMA_VERSION`
- **THEN** opening returns an explicit `WorldStoreError::VersionMismatch` error without executing migrations or modifying any files
