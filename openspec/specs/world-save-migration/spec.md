# World Save Migration

## Purpose
Sequential versioned schema migration ladder for SQLite save slots to support non-destructive upgrades across versions.

## Requirements

### Requirement: Sequential save slot schema migration
The game SHALL inspect `PRAGMA user_version` upon opening any existing save slot database file. If `user_version` is less than `WORLD_DB_SCHEMA_VERSION`, it SHALL execute sequential migration steps in ascending order up to `WORLD_DB_SCHEMA_VERSION`. If `user_version` exceeds `WORLD_DB_SCHEMA_VERSION`, it SHALL reject the file with `WorldStoreError::VersionMismatch`.

#### Scenario: Opening an earlier version executes sequential migrations
- **WHEN** opening a save slot database whose `user_version` is lower than `WORLD_DB_SCHEMA_VERSION`
- **THEN** all sequential migration steps run in order and `PRAGMA user_version` is updated to `WORLD_DB_SCHEMA_VERSION`

#### Scenario: Opening a newer database version is rejected
- **WHEN** opening a save slot database whose `user_version` is greater than `WORLD_DB_SCHEMA_VERSION`
- **THEN** opening returns an explicit `WorldStoreError::VersionMismatch` error without executing migrations or modifying the file
