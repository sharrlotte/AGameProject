## 1. Phase 0 — Baseline and guardrails

- [x] 1.1 Delete `src/symbol.rs`, `Symbol`, and `Symbols`; replace all block, item, and crafting keys with `String`
- [x] 1.2 Remove `lasso` and `serde_json` from `game/core/Cargo.toml`; delete every JSON reader and writer
- [x] 1.3 Add `RonDocument`, `Versioned<T>`, `read_document`, `write_document`, `atomic_write`, and `ron_document!`
- [x] 1.4 Add `ConfigState::Error(ConfigError)` and an error screen; replace config and mission `panic!` paths
- [x] 1.5 Delete `PowerConsumer`, `PowerStorage`, and the duplicate `#[cfg(debug_assertions)]`
- [x] 1.6 Add test modules and a `cargo test` baseline; keep checks to phase gates
- [x] 1.7 Document the schedule map from the design in code and `docs/architecture.md`
- [x] 1.8 Verify `grep` finds no `lasso`, `serde_json`, `Symbol`, or `Symbols` in `game/core`

## 2. Phase 1 — Ship grid and type consolidation

- [x] 2.1 Add `BlockPlacement`, `GridBlock`, `BlockRef`, `BlockOwner`, `ShipStats`, and `ShipBuildResult`
- [x] 2.2 Rewrite `ShipGrid` to `blocks: HashMap<Uuid, GridBlock>` plus `cells: HashMap<IVec2, Uuid>`
- [x] 2.3 Delete `BlockData`, `Block`, `BlockState`, `Blocks`, `ShipData`, `ShipState`, `ShipCalculatedStat`, `ShipCalculatedData`, `ShipFragment`, and `ComputedShipStats`
- [x] 2.4 Add `ShipMutation` and `apply_ship_mutations`; batch per ship and order removals, components, fragments, core death, stats, collider, event
- [x] 2.5 Implement `stats_from_grid`: core identity and position, bounds from extremes, mass, health, ranges, power, cargo, thrust
- [x] 2.6 Rebuild the collider from `ShipGrid.cells` only; remove `check_rebuild_ship_collider`
- [x] 2.7 Route ship spawn and editor placement through `rebuild_from_layout` and `validate_ship`
- [x] 2.8 Add grid, stats, mutation, and spawn-with-destruction integration tests
- [ ] 2.9 Remove every remaining `BlockData`/`Blocks` reference and confirm the game still spawns and destroys blocks

## 3. Phase 2 — Scene lifecycle and schedules

- [x] 3.1 Replace `SceneEntity` and `clean_state_item` with `DespawnOnExit` on all game, home, editor, pause, and settings roots
- [x] 3.2 Delete `SceneEntity`, the `Save` marker, `ShipState`, and manual cleanup loops
- [x] 3.3 Implement the `Input`, `Stats`, `Ai`, `Control`, `Weapons`, `Damage`, `Death`, `Present`, and `Cleanup` sets
- [x] 3.4 Chain damage → death → `ShipMutation` within the fixed tick; remove cross-frame message reliance
- [x] 3.5 Implement `PendingLoad` → `Loading` → `Game` load as a state transition
- [x] 3.6 Split `Pilot` and `FreeLook` control; add `F` follow toggle and middle-mouse pan
- [x] 3.7 Add the debug-only untagged-entity audit
- [x] 3.8 Add scene-exit entity-count, ordering, and input-mode tests
- [x] 3.9 Verify no gameplay entity survives leaving `Game` or loading a slot

## 4. Phase 3 — RON save v2

- [x] 4.1 Add `ShipDesign`, `WorldSave`, `ShipSnapshot`, `BlockSnapshot`, `BattleSave`, `CameraSave`, `DropSave`, and `RngSave`; set `SAVE_SCHEMA_VERSION = 2`
- [x] 4.2 Add `ShipCatalog`; load and save designs as RON; delete JSON schematics and `load_saved_schematics`
- [x] 4.3 Add starter `interceptor.ron` and `raider.ron`; copy them to `designs/` on first run
- [x] 4.4 Implement five-slot save/load UI in Home and Pause with timestamp, mission, ship count, and elapsed time
- [x] 4.5 Implement `CaptureSnapshot` and `RestoreSnapshot` for `WorldSave`, `ShipSnapshot`, `BlockSnapshot`, and `ItemDrop`
- [x] 4.6 Embed all ships in `WorldSave.ships`; remove `ship_ids`, `ShipSave`, `ships/*.ron`, and per-ship IO
- [x] 4.7 Save transforms, velocities, health, ammo, turret/pattern state, AI/runtime, battle timer/outcome, drops, camera, and RNG
- [x] 4.8 Exclude and clear projectiles, lasers, beams, damage text, and transient VFX on load
- [x] 4.9 Implement `.tmp` → `.bak` → rename writes and strict version rejection with UI errors
- [x] 4.10 Add design/embedded-world round-trip, version, atomic-write, slot, and starter tests
- [x] 4.11 Confirm no per-ship file is opened during save or load

## 5. Phase 4 — Power

- [ ] 5.1 Add single serde/component `PowerProducer { output }` and `EnergyConsumer { rate }`; delete old power types
- [ ] 5.2 Add `reactor` to `block.ron` and power fields to core, turrets, and thrusters
- [ ] 5.3 Implement equal-share production/demand/efficiency math and per-block `PowerEfficiency`
- [ ] 5.4 Apply efficiency to turret fire rate and thruster force/torque; disable features at `<= 0.01`
- [ ] 5.5 Add the power HUD and editor stats panel; update on change, not per frame
- [ ] 5.6 Add `feature_set!` and `make_ship_stat!`; migrate feature mapping and power/stat pipelines
- [ ] 5.7 Add power validation for output, rate, core producer, and missing producer warnings
- [ ] 5.8 Add formula, scaling, HUD, macro, and validation tests
- [ ] 5.9 Playtest core-only, reactor, deficit, and zero-production ships

## 6. Phase 5 — Targeting and combat

- [ ] 6.1 Target the enemy core for every ship strategy; reject targets without a core
- [ ] 6.2 Implement `Closest` tie-break by entity id and `MostHP` by current `ShipStats.health` then distance then id
- [ ] 6.3 Add `ProjectileOwner` and `Interceptable` to bullets and missiles; keep lasers non-interceptable
- [ ] 6.4 Implement `EnemyBullet` point defense with nearest/missile priority and lead prediction
- [ ] 6.5 Add `TurretState { target, aim_point, can_fire }`, `TurretTarget`, barrel-forward fire gate, and save-skipped targets
- [ ] 6.6 Parent continuous laser and pulling-beam visuals to the rotating barrel; fix the projectile laser axial offset
- [ ] 6.7 Guard empty barrels, apply pulling-beam pull once per ship, set damage text to layer 400, align missile z-order, reset timers
- [ ] 6.8 Scope all transient combat visuals to `SceneState::Game`
- [ ] 6.9 Add MostHP, point-defense bullet/missile/laser, empty-barrel, pulling-beam, damage-text, and laser-parent/direction tests
- [ ] 6.10 Playtest that the visible beam follows the barrel and matches the hitbox at several rotations

## 7. Phase 6 — AI profiles

- [ ] 7.1 Add `AiProfile`, `AiRule`, `AiCondition`, `BehaviorSpec`, `TargetRef`, and `PositionRef` with RON serde
- [ ] 7.2 Add `AiRuntime` and rule evaluation at `think_rate_hz` in `FixedUpdate` with priority and cooldowns
- [ ] 7.3 Replace the 11 `BehaviorX` markers, `ShipBehavior`, and `AIShipController` with one `AiBehavior` runtime enum
- [ ] 7.4 Implement all behavior semantics: idle, wander, patrol, guard, follow, regroup, kite, chase, attack, flee, kamikaze
- [ ] 7.5 Fix the flee direction, wander chances, attack target, and kamikaze blast; invalidate dead or out-of-range targets
- [ ] 7.6 Add editor AI panel: rule list, fields, validation, presets, preview label, and RON round-trip
- [ ] 7.7 Add AI profiles to starter designs and mission ships
- [ ] 7.8 Add flee-direction, priority/cooldown, reachability, target-invalidation, round-trip, and editor tests
- [ ] 7.9 Playtest each behavior from a rule

## 8. Phase 7 — Shared cargo

- [ ] 8.1 Add `cargo_capacity`, `cargo_used`, and `cargo_items` to `ShipStats`
- [ ] 8.2 Delete `Storage`, `StorageBuilder`, and `StorageState`; pool capacity from `StorageConfig`
- [ ] 8.3 Implement atomic pickup: full amount fits or the drop stays in the world
- [ ] 8.4 Implement player-only collection, nearest-ship magnet, and pickup radius
- [ ] 8.5 Recompute capacity on `ShipChanged` and handle storage destruction
- [ ] 8.6 Add cargo HUD and item breakdown in pause/editor stats
- [ ] 8.7 Add pooled-capacity, atomic-pickup, full-cargo, player-only, and capacity-loss tests
- [ ] 8.8 Playtest multiple storage blocks, a full ship, and enemy loot

## 9. Phase 8 — Performance and editor split

- [ ] 9.1 Replace 2,720 background sprites with a shader or two-to-three tiled parallax layers updated on camera movement
- [ ] 9.2 Pool 64 damage-text entities; reuse instead of spawn/despawn
- [ ] 9.3 Rebuild radar at most 10 Hz or on `ShipChanged`
- [ ] 9.4 Gate debug UI behind `DebugOverlay`; make ammo icons children or update them on change
- [ ] 9.5 Sort `BlockRegistry::iter` once at registration
- [ ] 9.6 Split `scene/editor.rs` into `mod`, `ui`, `input`, `ship_list`, `save`, `ai_panel`, and `grid`
- [ ] 9.7 Add layout constants, responsive panel widths, and remove scattered magic numbers
- [ ] 9.8 Debounce settings saves and merge with defaults
- [ ] 9.9 Add performance smoke tests and editor module tests
- [ ] 9.10 Apply remaining Appendix A review fixes not covered above

## 10. Verification and documentation

- [ ] 10.1 Update `docs/save.md`, `docs/combat.md`, `docs/ai.md`, `docs/architecture.md`, `docs/simulation.md`, `docs/config.md`, `docs/economy.md`, `docs/ui.md`, and `docs/defects.md`
- [ ] 10.2 Run `cargo fmt`, `cargo clippy`, `cargo check`, and `cargo test` at the final gate
- [ ] 10.3 Verify every acceptance scenario in the capability specs is covered by a test or a manual check
- [ ] 10.4 Verify no removed type, `lasso`, `serde_json`, `Symbol`, or `Symbols` remains
- [ ] 10.5 Run the manual playtest checklist: fresh install, battle, destruction, save/load, pause, editor, power HUD, AI, cargo, laser direction
- [ ] 10.6 Archive the OpenSpec change only after all tasks and manual checks pass
