## ADDED Requirements

### Requirement: Chunk persistence is integrated into WorldStore's chunks table
`WorldStore` SHALL expose `load_chunk(coord)`, `save_chunk(coord, data)`, and `remove_chunk(coord)` implemented by both `SqliteWorldStore` (backed by the `chunks` table inside `<data_dir>/saves/<slot_id>.db`) and `InMemoryWorldStore` (for `QuickBattle` and unit tests). Chunk payloads SHALL be encoded with `postcard` + `lz4_flex`, and unvisited chunks SHALL NOT create database rows.

#### Scenario: Single-file random-access chunk load and save inside the save slot database
- **WHEN** chunks at `(0, 0)` and `(100, -50)` are saved via `SqliteWorldStore` and the save slot `.db` is reopened
- **THEN** `load_chunk((100, -50))` deserializes only that chunk's row, returns the exact `ChunkData`, and `load_chunk((1, 1))` returns `Ok(None)`

#### Scenario: Empty chunk removes its row
- **WHEN** all enemies and debris are removed from a chunk
- **THEN** `remove_chunk(coord)` deletes its row from `chunks` so empty chunks do not consume storage

#### Scenario: InMemoryWorldStore round-trips chunks with zero disk I/O
- **WHEN** unit tests or `QuickBattle` save and load chunks via `InMemoryWorldStore`
- **THEN** chunk data round-trips accurately without touching disk
