## Context

The current registry is an enum (`BlockType`) compiled into the binary.
`block.ron` therefore cannot introduce new block ids without recompiling Rust,
and every save/selection/palette match statement knows the full enum. This
violates Slice goals of data-ness (adding a block = editing one `.ron` entry)
and produces spelling/duplication debt (`Meldown`, `LazerTurret`) that we only
wished to rename internally. Slice 2 lands after Slice 1 to keep the diff
linear and the review manageable.

## Goals / Non-Goals

**Goals:**
- String-keyed ids for `BlockType` and `ItemType`, interned at load time via
  `lasso::Rodeo` (or equivalent) in a `Symbols` resource.
- `block.ron` keyed by id; no `block_type` field in the schema; uniqueness
  enforced by the serializer/loader.
- `ItemRegistry` remains for items (ids are symbols), under the same pattern.
- Editor palette, drops table, selection map, spawn lookups move to symbol
  keys; save paths use the same symbols.
- Spec-driven breaking change: save format is intentionally incompatible.

**Non-Goals:**
- Ship-relative save redesign (Slice 6) — this slice only ensures the symbol
  ids are usable from it.
- Replacing embedded enums (`ShootingProjectile`, `TargetingStrategy`,
  `ShootPattern`) — those remain serde enums as runtime data on turrets/their
  components, and are intentionally not stringified.
- Symbol interning for non-registry strings (paths, texture names) — already
  plain strings; no benefit to interning them.

## Decisions

- **Lasso for interning, not a custom interner.** Crate is maintained,
  `Rodeo` fits, minimizes custom code; a bespoke interner would be more
  testable but is unnecessary at this scale.
- **`Symbol` newtype at the boundary, `&str`/`Symbol` inside hot paths.**
  Keep functions accepting `&str` where practical to avoid leaking interning
  details; intern at config load.
- **block.ron keyed map (data-driven key == id), one-of-a-kind.** Rejected
  alternatives: (a) keep the list and add a redundant `id` field — violates
  single source of truth; (b) heavy typed interned-id sections per def —
  unnecessary verbosity.
- **Embedded enums stay enums.** `ShootingProjectile` etc. are data on the
  component, not registry indices; stringifying them would reimplement serde
  badly and complicate turret deserialization for no practical win.
- **Breaking change on purpose**, via a `SYMBOL_REGISTRY_VERSION` gate in the
  loader error messages and a single migration note in `docs/defects.md`.
  Rejected: serde aliases / migrator — adds permanent debt for zero users.

## Risks / Trade-offs

- [Risk] Scattered compile errors on every match arm for `BlockType`. →
  Mitigation: single mechanical sweep per file, table of find/replace per
  arm, check per slice.
- [Risk] Runtime cost of interning at hot paths (spawn/destroy). → Cache id
  → `Symbol` lookups in the registry access pattern; hot loop is not on the
  critical path (spawn/destroy rates are low).
- [Risk] Old saves silently re-load as "no blocks found" — surprising for a
  dev mid-session. → Hard error with the word "incompatible" in `save/io.rs`.
- [Risk] Symbol table grows unboundedly for player-defined content. → Table is
  bounded by config size; not a concern here.

## Migration Plan

1. Merge `.ron` keyed-map blocks + mirror in `config/block.rs`; delete old
   `BlockType`/`BlockGroup` enums and all match sites; wire `Symbols`.
2. Update editor, drops, item registry, mission spawn, save/load error text.
3. `cargo check -p core`; confirm example fresh save and schema roundtrip.
4. Fresh start is acceptable for 0.1.0; `docs/defects.md` updated with the
   breaking-change note.

Rollback: revert the slice; no format migration tooling involved.
