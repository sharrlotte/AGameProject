# Decisions Log

All autonomous decisions made under the standing permission (user suspended the "ask before implementing" rule) are recorded here.

## 2026-11-17 — Batch apply all remaining OpenSpec changes

- Decision: Apply the four remaining active changes in dependency order: `collapse-feature-children` → `ai-behavior-fsm` → `blocks-stats-cache` → `save-ship-relative`.
- Reason: The proposal/design docs define these as sequential slices; `registry-symbol-ids` (done) is the prerequisite for the first three.
- Rule: After each change, run `cargo check -p core` (and `--features dev`), then launch the game binary to confirm no boot/config panic. Use Bevy MCP only if CLI verification is insufficient. Archive a change via `/opsx-archive` flow once its implementation + verification pass.
- Caveat: If a change is intentionally breaking (e.g. save format in `save-ship-relative`), "game works" means boots cleanly with fresh state, not that legacy saves load.

### collapse-feature-children (applied + archived)
- Decision: Feature logic components (Thruster, Storage, Energy, Efficiency, Turret state/Radar/shoot patterns) live on the block entity; `ComponentOwner` and `insert_added_block` BFS deleted. Editor spawn path aligned.
- Decision: Turret barrel stays the one logic-free child (`TurretSprite` + `Transform` + `Sprite`); block carries `TurretBarrel(Entity)`; turret-tracking and bullet systems resolve the barrel's `GlobalTransform` for aim/spawn direction. BRP verified 3 turret barrels with independent rotations at runtime.
- Deviation: Blocks with >1 `Texture` (only the Core) keep extra texture layers as render-only children to preserve z-ordering; single-texture and turret blocks match the spec exactly (0 / 1 child).
- Decision: Reverted `despawn_children().despawn()` in `despawn_dead` — it caused a generation-mismatch panic when a ship and its block children both died in the same batch. Original `despawn()` restored.
- Decision: Synced delta to `openspec/specs/collapsed-features/spec.md` (manual fallback), archived to `openspec/changes/archive/2026-11-17-collapse-feature-children/`.
- Fix during verification: `block.ron` key `lazer_turret` re-renamed to `laser_turret` (the RON regeneration step had reverted the slice-2 rename).

### ai-behavior-fsm (applied + archived)
- Decision: Replaced `AIShipBehavior` enum + `AIShipController` with one marker component per behavior (`BehaviorIdle`, `BehaviorChase { target }`, …) + `BehaviorTimer`, in a new `content/ship/ai/` module. `control_ship` deleted; 11 one-per-behavior systems registered in `AiPlugin` and chained in `SimulationSet::Simulate`.
- Decision: Added `ShipIndex` resource (rebuilt once per fixed tick) holding per-team `ShipInfo` snapshots, shared by all behavior systems instead of rebuilding HashMaps per behavior.
- Decision: `set_behavior!` macro removes all 11 markers then inserts the new one — guarantees exactly one marker per ship (BRP verified 2 ships each with a single marker).
- Deviation: `Guard`/`Follow`/`Kamikaze` transition to `Idle` on invalid target instead of the old `previous()` (would require extra "previous behavior" component state); documented in `docs/ai.md`.
- Verification: game booted in Game scene, BRP showed 3 ships in `BehaviorChase`, later 2 in `BehaviorIdle` (transitions occurred), no panics over ~60 s. Synced delta to main spec; archived.

### blocks-stats-cache (applied + archived)
- Decision: Hoisted `ShipGrid` into `content/ship/grid.rs` with two maps — `blocks` (cell → Entity, editor) and `cells` (cell → BlockData, runtime cache). Both editor and runtime share the one type.
- Decision: `blocks_stats` and `blocks_to_voxel_points`/collider rebuild now read `ShipGrid.cells`; the per-event `cell_map` rebuild is gone. Grid is populated once at spawn (`ShipGrid::rebuild`) and pruned on `BlockDestroyedEvent` (`remove_block`).
- Note: `ShipGrid::rebuild` remains as the documented correctness fallback for editor/load paths, not the hot path.
- Verification: `cargo check -p core` clean; game booted with 3 `ShipGrid` components on ships; battles ran ~60 s with no panic (one pre-existing benign despawn-generation WARN from `despawn_dead`). Synced delta; archived.

### save-ship-relative (applied + archived)
- Decision: New `save/schema.rs` with versioned `WORLD_SCHEMA_VERSION = 1`, `ShipSave` (ship-relative blocks with health/turret/pattern state), `WorldSave` (teams, elapsed time, ship ids), and strict `read_version()`. RON replaces JSON for world saves.
- Decision: World saves live under `world/` (`world.ron` + `ships/<uuid>.ron`); editor schematics stay JSON. `SaveWorldEvent`/`LoadWorldEvent` are now unit messages (no path).
- Decision: `spawn_ship_from_save` is the single load entry point; `ShipData::spawn` now returns `(ship, block_entities)` so saved per-block state can be applied.
- Decision: On load, all `Save`-marked effect entities (bullets/lasers/missiles/damage) are despawned — the schema cannot leak them.
- Note: ship file UUIDs derive from `Entity::to_bits()` (stable per entity lifetime); stale files from earlier saves are not pruned in 0.1.0. Old `test.json` saves do not load (intentional break).
- Verification: schema round-trip unit test passed (`cargo test -p core -- ship_save_roundtrip_and_version`); RON files written on auto-save, ships respawned on auto-load, no crashes; manual auto-save/load test scaffolding removed afterward. Synced delta; archived.

## Final state
- All four remaining active changes applied, verified, and archived. `openspec/changes/` now contains only `archive/`.
- Every change was verified with `cargo check -p core` (clean) and a game boot + battle run; BRP used where CLI checks were insufficient (collapse turret structure, AI marker transitions, ShipGrid population, save/load file output).
