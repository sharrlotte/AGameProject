## MODIFIED Requirements

### Requirement: Single battle save and campaign progress files
The save system SHALL store at most one active battle snapshot at `<data_dir>/saves/battle.ron` (with its companion single-file SQLite chunk store at `<data_dir>/saves/battle_chunks.sqlite` when chunk streaming is active) and one campaign progress document at `<data_dir>/saves/campaign.ron`. No `slot_*.ron`, per-ship `.ron`, or per-chunk `.ron` files SHALL be created.

#### Scenario: Battle save writes battle.ron and flushes battle_chunks.sqlite
- **WHEN** a battle is saved while unloaded enemies or debris exist in `ChunkStore`
- **THEN** `battle.ron` is written atomically, `battle_chunks.sqlite` is flushed, and no `slot_*.ron`, per-ship, or per-chunk `.ron` files are created

#### Scenario: Battle load restores live world and reattaches chunk store
- **WHEN** `battle.ron` is loaded
- **THEN** live entities are restored from `battle.ron` and `ChunkStore` remains attached to `battle_chunks.sqlite` so approaching unloaded chunks spawns their saved enemies and debris

#### Scenario: Battle clear removes both battle.ron and chunk store files
- **WHEN** `clear_battle_save` is called (such as on campaign victory) or a fresh non-loaded mission starts
- **THEN** `battle.ron` and `battle_chunks.sqlite` (including WAL/SHM sidecars) are cleared so stale chunks do not leak into a new battle
