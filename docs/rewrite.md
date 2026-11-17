# Rewrite Record — Branch `rewrite`

Records what changed across the rewrite, per slice. Deviations from the pre-existing
`/docs` specs are noted inline; anything marked **spec delta** amends the relevant doc.

## Slice 0 — build gate (`365301d`)

- `rust-toolchain.toml` pins `1.98.0`. The Windows `windows` 0.62.2 (D3D12) crate stack
  does not survive rustc 1.99.1's trait solver; 1.98.0 compiles it. This toolchain pin is
  required to build the project on this machine and on CI.
- `game/Cargo.lock` carries the unified Windows crate graph (windows/windows-core 0.62.2
  only) instead of the mixed 0.61.3/0.62.2 graph from HEAD. Without it `wgpu-hal` does
  not compile. **Spec delta:** `docs/architecture.md` should note the lock pin.
- `bevy-inspector-egui` and `bevy_brp_extras` are optional and only under the `dev`
  cargo feature; production builds contain no inspector. `bevy_hanabi` kept (intended for
  future effects).
- Empty modules `world/` and `ui/` deleted.
- `main.rs` inspector gated behind `all(debug_assertions, feature = "dev")`.

## Slice 1 — bevy 0.20 API port (`9c28145`)

HEAD did not compile against its own locked dependencies (`bevy` 0.20.1 / `rapier` 0.37).
Mechanical port applied per `docs/bevy-0.20-migration.md` (it was already on disk and is
kept as the migration record):

- UI: `Interaction` → `bevy::picking::hover::PickingInteraction`; deprecated `Button`
  removed everywhere; `BorderRadius` moved into `Node::border_radius`; `font_size` →
  `FontSize::Px(...)`.
- rapier: `Velocity { linear, angular }`; `Cuboid::new(Vec2::...)`; `intersect_shape`
  callbacks now `|entity, collider| -> bool`; `RapierConfiguration` requires
  `simulation_mode`/`num_threads`; the init requires
  `broad_phase_optimization_strategy`.
- `rand::Rng` → `RngExt`, combinator `.and()` → `.and_then()` / `.or()` → `.or_else()`.
- Spec delta: runtime behavior is unchanged (this was the pre-approved mechanical port).

## Slice 2 — P0 crash/panic removal (`50d1c48`)

- `save/io.rs`: panic-bearing `get_parent`/`get_id` rewritten to `Option` + `filter_map`
  with warn-and-skip; `save_dir` falls back to `./agameproject` with an error log
  (instead of `expect`); timestamp unwrap removed; typo `singel_shoot_pattern_query` fixed.
- `AmmoState::refill()` rewritten: respects `capacity - current`, requires **all**
  crafting items present, consumes atomically, returns bool, `error!` on impossible paths
  instead of panicking.
- Burst fire with empty ammo transitions to `Cooldown` — no more stuck `Shooting` state.
- `AIShipBehavior::Kiting` implemented (orbit target at ~`max_range * 0.9`, flee inward
  when too close); `Wander` can now transition into it — `todo!()` removed.
- `Parallax` turret's `PullingBeam` implemented: pulls enemy ships toward the shooter via
  direct velocity nudge (mass-independent, framerate-independent), with a beam sprite and
  `Visibility` toggle; registered in the bullet plugin.
- `rapier.single()` sites return early instead of `.expect()`; editor placement and
  `blocks_to_voxel_points` unwraps removed (warn + skip); `is_full` panic removed (clamp).
- `Chase self` panic → warn + `Idle`.

## Slice 2b — quality & structure (`a942015`)

- Efficiency system chain order fixed (`update_block_efficiency_base` resets first,
  then buffs apply); `exclude_sensors` removed (blocks **are** sensors); buffs are
  provider-team scoped; `ComputedShipStats::default().efficiency == 1.0`; ships outside
  the power map no longer keep stale efficiency.
- Stat computation macros generated the four duplicated systems
  (`update_thrust_power_stats`, `update_rotate_speed_stats`, `update_power_usage_stats`).
- `#[cfg(debug_assertions)]` gating: per-ship `ShipUIRoot` debug dump, FPS + camera-coords
  HUD, gameplay gizmos; diagnostics plugins gated to debug builds.
- `check_ship` deleted (top-up spawning is gone — see Slice 4); `SchematicCache`
  resource + `MAX_FACTIONS` removed; `utils::random_vec2_step` deleted with its last user.
- `TurretControl` removed; player turrets auto-target like AI turrets.
- Deploy wired: `DeployPayload.current_ship` is consumed by mission spawn as the player's ship.
- `cleanup_bullets` deleted; bullets/lasers/missiles/damage text/drops now carry
  `SceneEntity(Game)` — `clean_state_item` is the single scene-exit cleanup path.
- Drops collect at `DROP_PICKUP_RADIUS = 32 px` (was `sqrt(DROP_COLLECT_RADIUS)`).
- Laser fade clamp `>0.0`; laser beam pulse floor kept deliberately.
- `ShipUIRoot` damage numbers no longer spawn in release builds.
- `blocks_stats` adjacent row/column scan fixed (`y == size.y`, `x == size.x`);
  `ShipBehavior` left uncleared intentionally (AI re-issues at 15 Hz — documented).
- Editor Deploy consumes `current_ship`; stale `BlockButton`/`LAYER_BLOCK_OVERLAY` deleted.
- Selectable box-select: single push per entity; bbox test centered on `translation ± size/2`.

## Slice 3 — .ron config system (`c6d07dd`)

- Approved `docs/config.md` implemented: `RonAssetLoader<A>` in `config/loader.rs`
  (renamed from `ron.rs` to avoid clashing with the `ron` crate); `SceneState::Loading`
  boot gate with panic on missing/corrupt `game.ron`/`block.ron`; `setting.ron` falls
  back to defaults + warn; dev-only hot reload via the `dev` cargo feature
  (`bevy/file_watcher`); settings persist on change via save-on-change bridge with
  `set_if_neq` (no save/reload echo); all block defs moved to `block.ron`; gameplay
  constants moved to `game.ron`.
- `register_default_blocks` deleted; `register_default_items` kept (items deferred).
- `game/assets/` (empty, wrong root) deleted; `game/core/assets/config/...` is the
  live config path.
- See `docs/config.md` for the spec delta (module name, GameSetting location).

## Slice 4 — battle-core scaffolding (`6282f4e`)

- New `mission` module: `MissionDef` asset (`Asset`, `TypePath`, `Deserialize`,
  `deny_unknown_fields`); `BattlePhase` state (`Loading → Active → Ended`); `MissionDef`
  loaded from `assets/config/mission/quick-battle.ron`; `MissionPlugin` registered.
- Mission spawn order: Deployed `current_ship` if set → `ship_0` fallback → first
  available schematic. Enemies are `ship_1` and `ship_2` at fixed positions, team 1.
- Outcome: all enemy ships dead → Victory; all player ships dead → Defeat. Overlay
  (`BattleEndScreen`) with stats and **Retry**/**Home**. Retry re-enters Home → Game
  (full reset). `check_battle_outcome` counts only `Ship` entities.
- No top-up spawning; Quick Battle IS the Game mode in v1. Play and Quick Battle both
  enter it. **Spec delta:** `docs/mission.md` (new), `scenes.md`, `controls.md` updated.

## Slice 5 — CI / docs (this commit)

- `.github/workflows/build-games.yml`: `dtolnay/rust-toolchain` replaces `actions-rs`;
  cache targets `game/target` (was `game/core`, a source dir — ineffective); new
  `cargo check --locked` step before release build; key includes `game/Cargo.lock`.
- `clippy.toml` stale "errors in CI" comment corrected.

## Known-remaining work (out of scope of this rewrite)

- Mission select / campaign map / other objectives; rewards/progression.
- AI faction spawn tuning (`check_ship`-era random spawns are gone — missions will need
  roster design); Kamikaze/Follow/Regroup AI still never chosen by Rust code paths
  (Wander can pick Kiting; others are reachable only via `Guard`/`Follow` transitions).
- `drop_pickup_radius` lives as a code constant (32 px) — not in `game.ron`
  (kept minimal on purpose; can move when spec extends).
- Items still code-registered; `EfficiencyEffect`/`PulseUsage*`/`EnergyConsumer` power
  loop remains display-only plumbing.
- Save/load (`pause.rs test.json`) leaks missiles/lasers/drops on load (scene-entity
  cleanup covers most; the rest expire by `Lifetime`).
- Warnings: ~12 dead-code warnings remain (all pre-existing; several retire as the
  mission design fills in).
- Perf (Slice `blocks-stats-cache`): `blocks_stats` and collider rebuild now read a
  per-ship `ShipGrid` cache instead of rebuilding a `cell_map` from block data on each
  destroy event; the cache is populated once at spawn and pruned incrementally on
  block death, so the previous O(N²) burst path is O(cells near the damage).
- Slice `save-ship-relative`: world saves are now versioned RON (`world/` with `world.ron`
  + one `ships/<uuid>.ron` per ship) with ship-relative block data; the JSON `test.json`
  two-pass uuid→Entity linker and its `ShipSave/BlockSave/TurretSave/...` structs are deleted.
  `spawn_ship_from_save` is the single load entry point; `Save`-marked effect entities are
  despawned on load so no missiles/lasers/drops/damage text leak into a loaded world.
