## Context

Starter ships are data: `ShipDesign` RON documents embedded with `include_str!` in `save/design/lib.rs`, re-seeded into the user designs directory on every launch, loaded into `ShipCatalog`, and selected by name by the editor ship list and missions. Each design carries its own inline `AiProfile`; there is no `Role` component or global faction behavior. Triangle hull definitions (`wedge_1x1`, `plate_2x2`, `fin_1x2`, `spike_1x3`, `cap_1x1`) already support pointed silhouettes. Quick Battle currently spawns `Interceptor` for the player and two `Raider` enemies.

The change must stay additive: no save schema bump, no engine or AI changes, no new block definitions or art, consistent with the project's data-driven and separation-of-concerns rules.

## Goals / Non-Goals

**Goals:**
- Ship a bomber and a scout as built-in starters with clear, opposite strengths and weaknesses.
- Build both silhouettes from existing block definitions and art.
- Make both designs appear in the editor and showcase them in Quick Battle.
- Keep both designs structurally valid and power-feasible.

**Non-Goals:**
- A `Role` component, role stat modifiers, or any runtime type changes.
- New block definitions, textures, art generators, or palette regrouping.
- Save-format/schema version changes or migration code.
- New AI behaviors, conditions, or AI engine work.
- Balance tuning beyond the agreed moderate asymmetry.

## Decisions

1. **Role is data-only.** Role is expressed through block composition, turret choice and the inline `AiProfile`. Alternative considered: a `ShipRole`/`RoleModifiers` component with systems multiplying into `ShipStats` — rejected because it duplicates composition-derived stats and forces a schema version bump and save round-trip changes.
2. **Reuse existing blocks and art.** Flying wing via `plate_2x2`/`wedge_1x1`/`hull`; dart via `spike_1x3`/`fin_1x2`/`hull`. Alternative: new dedicated hull definitions with generated chunky-pixel art — rejected because it expands assets, validation and spec scope without improving role clarity.
3. **B2 mounts two `colaris`; Blackbird mounts one `laser_turret`.** `colaris` gives the longest existing base range (6000) and ammo-fed fire without power draw; the charge laser keeps the scout light with no ammo logistics. Alternatives: `spectre` (3×3 footprint hard to fit a wing), `foreshadow` (oversized for a starter), `swarmer` (3000 range would invert the bomber's reach advantage).
4. **Distinct stand-off versus skirmish AI.** B2: `Attack` initial, `Flee` at HP < 0.2 (distance 800, cooldown 8 s), `Kite(4500)` within 6000. Blackbird: `Chase` initial, `Flee` at HP < 0.4 (distance 1200, cooldown 5 s), `Kite(2500)` within 4000. Alternatives: both `Attack` (weak role contrast) or `Guard`/`Wander` (passive, may avoid combat).
5. **Seed as starters; Quick Battle enemies become B2 and Blackbird, player stays Interceptor.** This keeps a familiar baseline for comparison. Alternative: let the player fly a new ship — rejected because it removes that baseline.
6. **Spec delta, not engine change.** Only the `world-save` fresh-install seeding requirement changes; role content is captured in a new `starter-roles` capability. No requirement in `ai-profiles` or `editor-ui` changes because they already govern inline profiles and catalog-driven lists.

## Risks / Trade-offs

- [Multi-cell triangle footprints can overlap or disconnect the grid] → Plan cells on grid coordinates first, then run `validate_ship` and the starter validity tests; iterate with a dev run.
- [Laser charge plus many thrusters can exceed generation] → Size reactors and batteries from the computed demand; assert power feasibility in validation.
- [`colaris` ammunition depends on steel crafting, so the bomber can run dry] → This is part of its intended endurance weakness; ensure starting storage is sufficient and document the trade-off in the spec.
- [Triangle bevel highlights rotate with placement] → Existing accepted behavior from the `triangle-hull` change; no new art work.
- [Starter re-seeding overwrites locally edited copies] → Existing documented behavior; unchanged by this change.
- [Moderate asymmetry may still feel uneven in battle] → Tune only through `.ron` composition and AI thresholds, with approval, not through engine changes.

## Migration Plan

No runtime migration. Adding starter documents is additive and leaves existing saves and designs untouched. Rollback consists of deleting the two new starter `.ron` files, reverting the seed/test loops in `save/design/lib.rs`, reverting the Quick Battle roster, and reverting the spec-style docs; no save-data cleanup is required.

## Open Questions

None. Exact cell coordinates, grid envelopes and final per-block counts are implementation detail within the agreed silhouettes, loadouts and moderate asymmetry.
