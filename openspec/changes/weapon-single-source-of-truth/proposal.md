## Why

Weapon data was split across two parallel RON sources — simulation stats (`WeaponsConfig` / `WeaponSpecData`
in `game/assets/config/combat/weapons.ron`) and presentation profiles (`WeaponCatalog` / `WeaponProfile` in
`game/assets/config/player/weapons.ron`) — plus hardcoded Rust fallback defaults. The sources could drift,
missing presentation profiles degraded silently, and docs and HUD strings retained stale identifiers from an
older weapon roster.

## What Changes

- Introduce a single unified `WeaponDefinition` in the config foundation crate with a nested `stats` section
  (former `WeaponSpecData`) and a nested `presentation` section (former `WeaponProfile`), plus top-level
  cross-cutting weapon fields.
- Move the weapon RON to a single source at `game/assets/config/weapon/weapon.ron` (new domain folder layout).
- Delete the split types (`WeaponsConfig`, `WeaponCatalog`, `WeaponProfile`, `WeaponSpecData`) and rewire
  combat (stats), presentation (poses), and inventory (full definition + ammo) onto the unified definition.
- Delete hardcoded fallback defaults; the dev stage consumes the prod-loaded roster and the AnimTest stage
  provisions from the first roster entry.
- Decouple presentation from simulation components via events and presentation-scoped weapon state.

**BREAKING**: the weapon config asset path, its RON shape, and the inventory slot payload change.

## Capabilities

### New Capabilities

- (none)

### Modified Capabilities

- `weapon-roster`: a single unified weapon definition replaces the split sim/presentation rosters; the
  presentation catalog requirement that every sim roster id has a matching profile is superseded by one
  authored definition carrying both stats and presentation.
- `ron-config-loading`: weapon configuration ownership moves from the feature crate layer (`combat` owning
  `WeaponsConfig`) to the shared config layer (`config` owning the unified `WeaponDefinition`).
- `weapon-presentation`: presentation consumes the unified definition's presentation section instead of a
  separate weapon catalog.

## Impact

- Affected code: `engine/src/config/` (unified definition + registration), `engine/src/combat/weapon/`
  (inventory population from unified definition), `engine/src/presentation/anim/weapon/` (mount, pose,
  kick, sway, reload, swap from unified definition), `engine/src/presentation/ui/hud.rs` (weapon label from
  unified definition), and message layer for the new presentation-facing weapon state.
- Affected assets: `game/assets/config/` (new `weapon/weapon.ron`).
- Affected docs: `docs/01-combat-movement-and-weapons.md`, `docs/07-technical-architecture-save-and-roadmap.md`.

Unblocks: clean single-source weapon authoring, hot-reload of one weapon asset, and future roster growth
without dual-source edits.
