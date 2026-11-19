## ADDED Requirements

### Requirement: Chunk persistence is abstracted behind a ChunkStore trait
Chunk storage SHALL be accessed exclusively through a `ChunkStore` trait supporting `load(coord)`, `save(coord, data)`, `remove(coord)`, `total_enemy_count()`, `clear()`, and `flush()`. The crate SHALL provide `SqliteChunkStore` for production persistence and `InMemoryChunkStore` for unit tests.

#### Scenario: InMemoryChunkStore round-trips chunk data
- **WHEN** a `ChunkData` containing enemies and debris is saved to `InMemoryChunkStore` and loaded back by `ChunkCoord`
- **THEN** the loaded `ChunkData` matches the saved data and `total_enemy_count()` equals the sum of stored enemies

### Requirement: SqliteChunkStore persists chunks in a single WAL-mode SQLite file
`SqliteChunkStore` SHALL store all chunks in a single SQLite database file (`rusqlite` bundled, WAL journal mode) with `PRAGMA user_version` schema validation and a `chunks(x INTEGER, y INTEGER, enemy_count INTEGER, data BLOB, PRIMARY KEY(x, y))` table. Chunk payloads SHALL be serialized with `postcard` and compressed with `lz4_flex`. Unvisited chunks SHALL NOT create database rows.

#### Scenario: Single-file random-access chunk load and save
- **WHEN** chunks at `(0, 0)` and `(100, -50)` are saved to `SqliteChunkStore` and the database is reopened
- **THEN** loading `(100, -50)` deserializes only that chunk's row, returns the exact `ChunkData`, and loading an unvisited chunk `(1, 1)` returns `Ok(None)`

#### Scenario: Indexed enemy_count query without deserializing blobs
- **WHEN** multiple chunks containing enemy ships are saved in `SqliteChunkStore`
- **THEN** `total_enemy_count()` returns the exact total number of stored enemies via SQL aggregation without deserializing chunk blobs

#### Scenario: Empty chunk removes its row
- **WHEN** all enemies and debris are removed from a chunk
- **THEN** its row is deleted from `chunks` so empty visited chunks do not consume storage

#### Scenario: Schema version mismatch fails loudly
- **WHEN** `SqliteChunkStore` opens a database whose `PRAGMA user_version` does not match `CHUNK_SCHEMA_VERSION`
- **THEN** opening returns an explicit `ChunkStoreError::VersionMismatch` error and does not silently overwrite the database
