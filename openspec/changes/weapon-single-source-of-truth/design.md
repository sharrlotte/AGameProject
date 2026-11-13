## Context

The weapon data was unified once before (`2026-11-13-collapse-to-two-crates`): a single `WeaponsConfig`
with a unified `WeaponDef` in `game/assets/config/combat/weapons.ron`, deleting the presentation-only
catalog file. The later weapon presentation effort (`2026-11-13-weapon-grip-ik` era) re-introduced a split —
presentation `WeaponCatalog` / `WeaponProfile` in `game/assets/config/player/weapons.ron` alongside sim
stats in `WeaponsConfig` at `game/assets/config/combat/weapons.ron`.

The split is the source of drift: two hand-synced RON files, hardcoded Rust fallback defaults duplicating
authored values, stale documentation (`docs/01-combat-movement-and-weapons.md` still lists pre-rename weapons),
and a hardcoded HUD string.

Constraints:

- Bevy 0.20, assets under `game/assets/`, `assert_ron_config_parses!` resolves via `CARGO_MANIFEST_DIR`
  ancestors.
- Keep source files under 500 lines; sub-features use 1-word file names.
- Feature-gate parity required for `dev` / `AnimTest` under `--no-default-features`.
- Persistence is still a stub; no save migration is in scope.

## Goals / Non-Goals

**Goals:**

- Establish a single source of truth for weapon data.
- Delete hardcoded fallback defaults that duplicate authored values.
- Move dev / AnimTest provisioning onto the prod-loaded roster.

**Non-Goals:**

- No gameplay, balance, or visual behavior changes.
- No new weapons or attachment system work.
- No persistence implementation.

## Decisions

### D1. Unified `WeaponDefinition` in the config foundation

- **Choice:** One `WeaponDefinition` type in the config foundation crate owns all weapon data: nested
  `stats` (former `WeaponSpecData`) and nested `presentation` (former `WeaponProfile`), plus top-level
  cross-cutting weapon fields.
- **Rationale:** One concept, one owner, one RON file; naming reflects the unified nature.
- **Alternative considered:** Keeping two structs joined by id at runtime. Rejected — still leaves two
  sources of truth.

### D2. Nested structure with top-level cross-cutting fields

- **Choice:** `stats` and `presentation` are nested; cross-cutting fields (muzzle offset, swap cooldown,
  stance hold) live at the top level.
- **Rationale:** Grouping authored data; cross-cutting concerns are shared, not purely visual.

### D3. Single RON asset path

- **Choice:** `game/assets/config/weapon/weapon.ron` — one weapon file in a new domain folder.

### D4. Delete fallback defaults

- **Choice:** Remove `Default` implementations and serde fallbacks that duplicate authored values; dev
  consumes the loaded roster.

### D5. Dev and test-stage provisioning

- **Choice:** Dev stage consumes the prod-loaded weapon config; the AnimTest stage provisions from the
  first roster entry.

### D6. Presentation decoupling

- **Choice:** Presentation consumes weapon state through events and a presentation-scoped weapon state
  rather than querying simulation components directly.

### D7. Inventory payload

- **Choice:** `WeaponInventory` slots hold the full unified `WeaponDefinition` plus ammo.

## Risks / Trade-offs

- [Risk] The merged definition grows complex; validation must cover stats, presentation, and cross-cutting fields. → Extend catalog validation and RON parse tests.
- [Risk] Hot reload becomes a single weapon asset. → Reuse the existing `RonConfig` hot-reload machinery.
- [Risk] Dev-stage wiring depends on prod config loading. → Reuse the standard config registration path.

## Migration Plan

1. Add unified `WeaponDefinition` with nested `stats` / `presentation` and top-level cross-cutting fields; add `RonConfig` impl and `game/assets/config/weapon/weapon.ron`.
2. Rewire combat systems to consume stats from the unified definition; rewire presentation mount/pose/kick/sway/reload/swap systems to consume the presentation section.
3. Change `WeaponInventory` slots to hold the unified definition plus ammo; delete split types and fallback defaults.
4. Wire dev stage and AnimTest to the prod-loaded roster (first entry).
5. Decouple presentation from simulation components via events and presentation-scoped state.
6. Update docs and specs.

## Open Questions

- None remaining; all design decisions are confirmed.
