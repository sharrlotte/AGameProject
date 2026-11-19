## MODIFIED Requirements

### Requirement: Single battle save and campaign progress files
The save system SHALL store each playthrough as a single self-contained SQLite save slot at `<data_dir>/saves/<slot_id>.db`. No `battle.ron`, `campaign.ron`, global `fleet.ron`, `slot_*.ron`, or per-ship `.ron` save files SHALL be created for save slots. Global user preferences SHALL remain in `<data_dir>/settings.ron`.

#### Scenario: Save slot is a single self-contained SQLite file
- **WHEN** an open-world save slot is created or saved
- **THEN** all world metadata, active entities, and per-slot designs are written into `<data_dir>/saves/<slot_id>.db` and no `battle.ron` or `campaign.ron` is created

#### Scenario: Loading a save slot reads only its database file
- **WHEN** a save slot is loaded from the Save Slot Picker
- **THEN** only that slot's `<slot_id>.db` file is opened

### Requirement: Saves are versioned and atomic
Every save slot database SHALL carry `PRAGMA user_version = WORLD_DB_SCHEMA_VERSION`. The loader SHALL reject a version other than the current schema version with an error naming expected and found versions. All save writes SHALL commit inside an ACID SQLite transaction so a failure leaves the previous committed state intact.

#### Scenario: Version mismatch fails cleanly
- **WHEN** an incompatible `.db` version is opened
- **THEN** loading returns an incompatibility error and does not spawn a partial world

#### Scenario: Failed transaction preserves previous committed save
- **WHEN** a write transaction fails or rolls back
- **THEN** the previously committed state in `<slot_id>.db` remains intact and readable

### Requirement: Hard cutover with no fabrication
The loader SHALL NOT import legacy JSON schematics, v1–v7 `battle.ron`, `campaign.ron`, or `fleet.ron`, fabricate phantom ships, or repair invalid ships. Legacy files SHALL remain on disk untouched.

#### Scenario: Legacy-only installation
- **WHEN** only legacy `.ron` or `.json` save files exist on disk
- **THEN** the Save Slot Picker shows no loadable slots and offers `New World` without deleting or migrating legacy files
