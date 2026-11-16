## Why

`BlockType`, `ItemType`, and `BlockGroup` are enums whose variants double as
registry keys: a new block type in `blocks.ron` forces a Rust recompile, and
save files serialize variant names verbatim (`"Meldown"`, `"LazerTurret"`).
Keying the registry by interned symbols fully decouples data from compile-time
code and makes the `.ron` authoring story real end-to-end.

## What Changes

- Replace `BlockType`, `ItemType`, `BlockGroup` registry-key enums with
  interned `Symbol` ids (lasso interner) for blocks, items, and palette
  groups; embedded enums (ShootingProjectile, TargetingStrategy, ShootPattern)
  stay serde enums as plain data.
- `blocks.ron` becomes keyed: `{ swarm: (…), hull: (…), … }` with uniqueness
  enforced by construction; `blocks.rs` mirror types produce `BlockRegistry`
  keyed by symbol; the same map is reused by drops/spawn paths.
- `BlockRegistry` exposes `get(id: &str)` / symbol-based lookup helpers;
  `.ron` no longer carries a `block_type` field (its key is the id).
- **BREAKING**: existing `.json` schematics and world saves keyed by variant
  name will not load. No migration provided — save format breaks intentionally
  as a prerequisite to the Slice 6 ship-relative schema.
- Editor palette, drop tables, item registry, teams' block accessors update to
  symbol-keyed lookups.

## Capabilities

### New Capabilities
- `symbol-registry`: string-keyed, interned block/item registry wired into
  `.ron`, editor, drops, and save path (with save breakage declared).

### Modified Capabilities
<!-- No existing specs; this builds on foundation-simulation-sets landing first. -->

## Impact

- Code: `content/block/components.rs` (BlockType/BlockGroup/registry),
  `content/item/*`, `config/blocks.rs`, `config/mod.rs`, editor selection
  types, drops, `save/io.rs` (error messages), `mission/state.rs`, `docs/
  config.md`, `docs/assets.md`, `docs/defects.md`.
- Dependencies: add `lasso`.
- Old `.json` schematic/save formats become invalid; fresh start acceptable
  per 0.1.0 project guidance.
