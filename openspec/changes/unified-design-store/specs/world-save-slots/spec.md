## MODIFIED Requirements

### Requirement: Per-slot ship designs with Home sandbox isolation
When inside an active save slot, `ShipCatalog` and the Ship Editor SHALL read and write that save slot's `designs` table through `SlotDesignStore`. When launched from `Home`, the Ship Editor and `QuickBattle` / `Skirmish` SHALL use `FileDesignStore` backed by `<data_dir>/sandbox_designs/*.ron` without touching any save slot `.db`. All design writes to a save slot SHALL commit atomically in a single SQLite transaction, and `ShipSavesChanged` SHALL only be emitted after persistence succeeds.

#### Scenario: Saving a design inside a save slot isolates it to that slot
- **WHEN** the player opens the Ship Editor from Pause inside Slot A, saves a modified ship design, and later opens Slot B
- **THEN** Slot A contains the modified design and Slot B's `designs` table is unaffected

#### Scenario: Home Ship Editor and QuickBattle do not touch save slots
- **WHEN** the player edits a ship from Home or plays `QuickBattle`
- **THEN** no file in `<data_dir>/saves/*.db` is created, modified, or deleted

#### Scenario: ShipSavesChanged is only emitted after persistence completes
- **WHEN** a design save is requested
- **THEN** `ShipSavesChanged` is emitted only after `DesignStore::save` succeeds on the underlying store
