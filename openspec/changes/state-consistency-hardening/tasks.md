## 1. World store and schema v2
- [ ] 1.1 Bump `WORLD_DB_SCHEMA_VERSION` to 2 and reject any `PRAGMA user_version` that differs in either direction with `WorldStoreError::VersionMismatch`
- [ ] 1.2 Remove `rng_blob` from `WorldMeta` and from the encode/decode sites in `save/world/system.rs`
- [ ] 1.3 Make `meta.seed` authoritative for slot listing, world loading, and `GameRng` initialization; set `GameRng` on create, load, and quick-battle paths
- [ ] 1.4 Add `list_slots()` to the `WorldStore` trait and implement it for `SqliteWorldStore` and `InMemoryWorldStore`
- [ ] 1.5 Add a unit test for schema rejection in both directions and a round-trip test proving the listed seed equals the restored `GameRng` seed

## 2. Validation and snapshot integrity
- [ ] 2.1 Add `validate()` with a typed `InvalidData` error to `ShipDesign`, `ShipSnapshot`, `DebrisSnapshot`, and `BlockSnapshot`
- [ ] 2.2 Call `validate()` immediately after decode in `read_designs_dir`, SQLite design decode, and `load_world_data`; reject the whole load on structural errors and show a notice naming the entity and reason
- [ ] 2.3 Replace `BlockSnapshot`'s four pattern options with `PatternSave { Single, Burst, Laser, Pulling }` and apply exactly one pattern state
- [ ] 2.4 Normalize clampable fields (charge, cooldown, aim point, cargo totals, remaining life) at apply time with a warning instead of rejecting the load
- [ ] 2.5 Reject duplicate `ShipSnapshot.id` values and dangling ship references loudly instead of silently overwriting `ship_map` or degrading tactics
- [ ] 2.6 Add unit tests for overlapping cells, duplicate ids, invalid team ids, multiple patterns, and cargo normalization

## 3. World session intent and Play flow
- [ ] 3.1 Replace `PendingLoad(bool)` with `PendingWorld { None, Restore { slot_id } }` in `resource/mod.rs` and update all writers/readers
- [ ] 3.2 Implement last-save resolution (active `SaveSlot` first, else max `updated_at_ms` from `list_slots()`) in a single reusable handler
- [ ] 3.3 Make Play on Home and in the editor use the shared handler; when no slot exists, create a generated-name slot with starter designs and enter it
- [ ] 3.4 On validation failure, show a notice and leave scene/session/pending unchanged; never auto-create or overwrite
- [ ] 3.5 Add a Home "Worlds" button that opens the slot picker; remove picker opening from Play
- [ ] 3.6 Fix the editor nav label ("Resume Game" / "Back to Home") by `EditorReturnTarget` and keep Back's current restore/Home behavior
- [ ] 3.7 Verify `QuickBattle` never writes to `<data_dir>/saves/*.db` and add a regression test with a temp data dir
- [ ] 3.8 Add tests for active-slot priority, most-recent fallback, auto-create, and failed-continue staying put

## 4. Modal stack
- [ ] 4.1 Add `ModalStack`/`Modal` in `ui/` and remove `PauseState`, `SettingState`, `SlotPickerState` from `ui/state.rs` and all `init_state` calls
- [ ] 4.2 Make Escape pop the top modal, or toggle Pause when empty in Game/ShipEditor; gate the handler on Settings/SlotPicker not being on top
- [ ] 4.3 Clear the stack and unpause `Time<Virtual>` on `OnExit(SceneState::Game)`; drive virtual-time pause from the top modal only
- [ ] 4.4 Replace every `run_if(in_state(PauseState|SettingState|SlotPickerState))` gate with a `modal_is(Modal)` predicate
- [ ] 4.5 Replace modal `DespawnOnExit<State>` usages with explicit despawn-on-change and update `DebugAuditPlugin` for the new ownership
- [ ] 4.6 Add tests for Escape closing Settings before Pause, scene exit clearing modals, and no unpause when SlotPicker is open on Home

## 5. Team and component invariants
- [ ] 5.1 Rewrite `Team` as a private bounded id with `TryFrom<u8>` deserialization, computed `color()`, and no `Reflect`; remove `assert!` from construction
- [ ] 5.2 Make `Skill`, `Throttle`, and `BoundingBox` fields private with validated setters/accessors
- [ ] 5.3 Collapse `TurretState` into a single runtime-state enum and remove dead `shoot_at` and `LaserBeamPatternState.targets`
- [ ] 5.4 Derive cargo `used` from item counts and capacity; remove the independently writable total
- [ ] 5.5 Clean up stale beam/entity handles on despawn and reconcile `Brain.shed_boosters` with `Disabled` markers
- [ ] 5.6 Reset all `EditorState` fields on `OnEnter(SceneState::ShipEditor)` and replace `SlotPickerUi`'s delete bool with `DeletePrompt { None, Confirming(Uuid) }`

## 6. Grid, health, and config compliance
- [ ] 6.1 Make `ShipDesign::spawn_with_id` fallible with a typed error and call `BlockRegistry::validate_ship` inside it
- [ ] 6.2 Make `BlockGrid.blocks`/`cells` private and rebuild `cells` only from `blocks` in one method
- [ ] 6.3 Add `sync_ship_health` in `SimulationSet::Stats` before AI so `ShipStats.health` always equals the sum of block health
- [ ] 6.4 Extend config validation to `ShootPattern` and `Thruster` numeric fields per `config-validation`; reject negative damage/width/lifetime with the config error screen
- [ ] 6.5 Add tests for spawn rejecting overlapping layouts, health tracking partial damage, and negative projectile config

## 7. Documentation and verification
- [ ] 7.1 Update `/docs` specifications for save schema v2, Play/Worlds semantics, modal ownership, and validation behavior (spec only, no notes)
- [ ] 7.2 Run targeted unit tests (`cargo test`) for all touched modules
- [ ] 7.3 Run `cargo dev` in the background and verify Play create/continue, Worlds picker, Escape layering, and load rejection in-game
