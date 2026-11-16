## Why

Every block spawns several child entities just to carry feature components
(sprite, turret, thruster, storage, energy, efficiency): one block costs 4+
entity IDs, and the entire `ComponentOwner` / `insert_added_block` / child
BFS dance exists only to reconstruct ownership chains and copy components
upstream later. The engine is already moving toward data-on-components;
children only existed for independent transforms, which only the turret sprite
actually needs.

## What Changes

- Features become components directly on the block entity: `Texture`,
  `Thruster`, `Storage`, `EnergyConsumer`, `EfficiencyProvider` lose their
  child entities. `ComponentOwner` and the BFS-insert pass are deleted.
- `Turret` becomes a component on the block with an owned child **only** for
  its rotating sprite (the barrel pivot). Unrelated access (team, health,
  `BlockOwner`) collapses onto the same entity.
- Spawn paths collapse from `Feature::spawn` into `Feature::insert`, taking
  the block entity directly.
- Rendering split: block sprite rendered as one entity; turret barrel sprite
  as a small child component pair (`TurretSprite` + its own child entity with
  `Transform` only). Editor preview path shrinks accordingly.
- Entity count per 12-block ship: from ~50 to ~25. **BREAKING** internal
  structure — no schema/format change.

## Capabilities

### New Capabilities
- `collapsed-features`: feature components live directly on the block entity,
  with the turret sprite as the one deliberate child-entity exception.

### Modified Capabilities
<!-- No existing specs (this builds on foundation-simulation-sets landing first). -->

## Impact

- Code: `content/block/component.rs` (BlockData spawn/insert),
  `content/block/feature/*/` (collapse patterns), `content/block/system.rs`,
  `content/block/feature/system.rs`, `scene/editor.rs` (spawn_placed_block),
  `save/io.rs` (no structural change required for schema), `docs/architecture.md`,
  `docs/simulation.md`.
- No dependency changes, no schema changes; runtime behaviour unchanged.
