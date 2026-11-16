# Known Defects (current implementation)

Factual defects found in the current code, used as a work queue. Not a changelog.

## P0 — crash / panic paths

1. `cargo check -p core` fails inside `wgpu-hal` — **fixed (Slice 0)**: pinned
   `rust-toolchain.toml` to 1.98.0 + carried over the unified `windows` graph lock refresh
   (windows 0.62.2 only). `cargo check -p core` passes on this machine.
2. `AIShipBehavior::Kiting => todo!()` — **implemented (Slice 2)**: kiting AI holds
   weapon range and orbits the nearest enemy while turrets engage; reachable via `Wander`.
3. `AmmoState::refill()` — **rewritten (Slice 2)**: respects `capacity - current`, requires
   all crafting items present, consumes atomically, never panics.
4. Burst fire with empty ammo → stuck `Shooting` — **fixed (Slice 2)**: transitions to `Cooldown`.
5. Panic-bearing production paths — **fixed (Slice 2)**: `save_dir`, save handlers
   (`get_id`/`get_parent`), timestamp fallback; `rapier.single()` sites (bullet ×2, radar,
   efficiency) return early; editor placement unwraps and `blocks_to_voxel_points` unwrap →
   warn + skip/false; `storage is_full` panic → clamp; `Chase self` panic → warn + Idle.

## P1 — broken features / leaks / heavy paths

6. Efficiency buffs do nothing — **fixed (Slice 2b)**: base reset runs first in the chain,
   `exclude_sensors` filter gone, buffs are provider-team-scoped.
7. Background parallax sprites have no `SceneEntity(Game)` tag — **fixed (Slice 2b)**:
   parallax layers and damage numbers now tagged (fixed in Slice 2b background section).
8. `check_ship` parses all schematics from disk every 100 ms — **fixed (Slice 2b)**:
   `SchematicCache` resource; reload on `OnEnter(Game)`; faction count is `MAX_FACTIONS`.
9. Pause → Load leaks missiles, lasers, drops, damage text — **fixed (Slice 2b)**:
   bullets/lasers/missiles/damage text/drops now carry `SceneEntity(Game)`, and drops
   collect at `DROP_PICKUP_RADIUS = 32 px` instead of `sqrt(DROP_COLLECT_RADIUS)`.
10. `update_ship_ui` rebuilds a `Text2d` debug dump every fixed tick — **fixed (Slice 2b)**:
    `ShipUIRoot` + `update_ship_ui` are `#[cfg(debug_assertions)]`-only; FPS/camera-coords
    HUD and gameplay gizmos gated the same.
10b. `handle_pulling_beam` unfinished/inert — **implemented (Slice 2)**: Paralax beam pulls
    enemy ships toward the shooter (direct velocity nudge), with beam visual.
11. Laser fade `.max(0.5)` never reaches transparency — **fixed (Slice 2b)**: clamp to 0.0.
    Laser beam pulse floor kept at 0.5 deliberately.
12. `ShipBehavior` never removed — **accepted**: AI re-issues every 66 ms; behavior flips
    are invisible on screen. Documented as known debt.
13. `cleanup_bullets` duplicated `clean_state_item` — **fixed (Slice 2b)**: deleted;
    everything combat-spawned now carries `SceneEntity(Game)` and `clean_state_item` is
    the single cleanup path.

## P2 — wrong behavior / inconsistent data

14. `blocks_stats` never scans the directly adjacent row/column — **fixed (Slice 2b)**:
    `y == size.y` / `x == size.x` now scanned; `+1` probes deleted.
15. `ComputedShipStats::default().efficiency == 0.0` → first-frame thrust of 0 — **fixed
    (Slice 2b)**: default is `1.0`; ships outside the power map reset to `1.0` each tick.
16. `EnergyConsumer`/`PowerUsage*`/`EfficiencyEffect` — **unchanged**: built, wired, still
    unused by any placed block; kept as plumbing (config spec may wire it later).
17. `AmmoState::refill` inverted — **fixed (Slice 2)**; `AmmoBuilder::build` now returns
    a plain `Ammo` instead of an always-`Some` `Option` — **fixed (Slice 2b)**.
18. Efficiency sign inconsistency (power divided, thrust multiplied) — **accepted**:
    power usage and thrust are different stats; a power-rich ship benefits, a thrust
    ship doesn't. config.md may unify later.
19. `TurretControl::Manual/Hold` — **removed entirely (Slice 2b)**: turret control enum
    and the manual/hold branches are gone; turrets always auto-target the camera-range
    enemies.
20. `selectable` box-select double-push + mis-centered box — **fixed (Slice 2b)**: single
    push; bbox test uses `size/2` extent around the translated center.
21. Editor zoom `[0.1, 5.0]` vs game `[0.2, 30.0]` — **unchanged**: editor zoom is
    intentionally tighter for building (documented). `block-steel.PNG` case mismatch —
    **open**; `colaris.png` duplication — **open**; `Meldown`/`"steal"` typos —
    **fixed (Slice 2)**: ids renamed to `meltdown`/`steel`; `singel_shoot_pattern_query`
    typo — **open**.
22. Drops collected at `sqrt(DROP_COLLECT_RADIUS)` — **fixed (Slice 2b)**: `DROP_PICKUP_RADIUS`.
23. `max_range = 1000`, `min_range = 10000` magic seeds — **unchanged** (documented tuning
    debt; config spec will absorb).
24. Pause → Load leaks — **fixed (Slice 2b)**: same entities now die on scene exit;
    `test.json` slot — **open** (campaign/mission work will replace); camera follow
    reset — **open**; ship list rebuild 200 ms — **open** (accepted).
25. `handle_block_destroy` duplicate events on multi-step accumulations — **open** (rare,
    harmless duplicate drops by non-players; accepted).
26. Phase-zero AI — **partially fixed (Slice 2)**: `Wander` can now transition to `Kiting`
    (reachable); `draw_indicators` still draws for all ships — **open**.

## P3 — structure / hygiene

27. No `.ron` data files anywhere — block/item defs were Rust literals; `check_ship`
    hardcoded two teams — **blocks fixed (Slice 3)**: all block defs and gameplay constants
    moved to `assets/config/base/block.ron` and `game.ron`, loaded through the approved
    boot-gated config system with dev-only hot reload; `check_ship` still uses
    `MAX_FACTIONS = 2` (design-level choice, not data-driven in v1). Items are still
    hardcoded (`register_default_items`) — deferred per `docs/config.md`.
28. Dev-only code not usable in prod builds but linked: `bevy-inspector-egui` (main
    dependency), `bevy_brp_extras` (optional), `HanabiPlugin` (no effects), empty `world/`
    and `ui/` modules.
29. `.github/workflows/build-games.yml` builds but never runs check/test/clippy;
    cache path `game/core` (source dir) instead of `game/target`; `actions-rs/toolchain`
    is retired; no `docs/` mentioned in CI.
30. `clippy.toml` comment claims CI treats warnings as errors — it doesn't.
31. `/docs` was missing; this file and its siblings now exist.
32. Debug dumps in production: FPS+camera-coords text (`FpsText`), `ShipUIRoot`, damage
    numbers, gizmos in `update_ship_behavior`.
33. Typo/ergonomic debt: `Meldown`, `"steal"`, `singel_shoot_pattern_query`,
    `from_bits_truncate` in comment area (index 31+ reserved), `'How does this event
    happend'` message.
34. Redundant/legacy: `cleanup_bullets` (duplicates scene cleanup), `Meldown` naming,
    repeated 12× identical block registration boilerplate, duplicated stat-update logic
    between `energy` and `thruster` (macro candidate), manual `impl Default for
    TurretState`, manual `PartialEq` on `Team` (could derive on id), ambient
    `GameVars` knob (deleted) with no input binding.

## Slice 2 (registry-symbol-ids) — breaking change

- World saves and schematics written against the pre-symbol enum-name schema (`.json`
  files under `schematics/`) are **incompatible** and will fail to load with an
  "incompatible"/schema error instead of producing an empty or default ship. No
  migration is provided; a fresh start is expected for 0.1.0. `block.ron` is now a
  keyed map (`{ hull: (...), ... }`) keyed by interned `Symbol` id; duplicate ids
  are rejected at boot.
