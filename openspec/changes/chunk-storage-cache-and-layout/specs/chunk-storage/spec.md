## MODIFIED Requirements

### Requirement: Chunk persistence is integrated into WorldStore's chunks table
`WorldStore` SHALL expose batched and partial chunk persistence (`load_chunk`, `load_chunks`, `save_chunk_entry`, `save_chunks_batch`, `remove_chunk` taking `&self` with interior mutability) implemented by both `SqliteWorldStore` (backed by the `chunks` table inside `<data_dir>/saves/<slot_id>.db`) and `InMemoryWorldStore` (for Skirmish and unit tests). The `chunks` table SHALL store `(x, y)` primary key, durable `visited` flag, and per-subsystem nullable `BLOB` columns starting with `bodies_blob` (`postcard` + `lz4_flex` encoded `ChunkBodies { enemies, debris }`). Unvisited chunks with no stored data SHALL NOT create database rows; once a chunk has `visited = 1`, clearing its enemies and debris SHALL set `bodies_blob = NULL` while preserving its row and `visited = 1` flag.

#### Scenario: Single-file random-access chunk load and save inside the save slot database
- **WHEN** chunks at `(0, 0)` and `(100, -50)` are saved via `SqliteWorldStore` and the save slot `.db` is reopened
- **THEN** `load_chunk((100, -50))` deserializes only that chunk's row in one primary-key lookup, returns the exact `ChunkEntry` (`visited` + `bodies`), and `load_chunk((1, 1))` returns `Ok(None)`

#### Scenario: Emptying a visited chunk clears bodies_blob without erasing visited state
- **WHEN** all enemies and debris are loaded out of a visited chunk `(2, -1)`
- **THEN** its row in `chunks` retains `visited = 1` with `bodies_blob = NULL` instead of deleting the row

#### Scenario: Partial column update leaves future subsystem columns untouched
- **WHEN** only `bodies_blob` is dirty for chunk `(2, -1)`
- **THEN** `SqliteWorldStore` updates `bodies_blob` without requiring re-encoding of untouched subsystem columns

#### Scenario: InMemoryWorldStore round-trips chunks with zero disk I/O
- **WHEN** unit tests or Skirmish save and load chunks via `InMemoryWorldStore`
- **THEN** chunk entries (`visited` and `bodies`) round-trip accurately without touching disk
