## 1. Config and data model

- [x] 1.1 Add `#[serde(default)] default_fleet: Vec<String>` and `fleet_spacing: f32` to `GameConfig`, its `Default` impl, and `validate()` (non-negative spacing)
- [x] 1.2 Set `default_fleet: ["Interceptor", "B2"]` and `fleet_spacing: 400.0` in `core/assets/config/base/game.ron`
- [x] 1.3 Replace `DeployPayload.current_ship: Option<ShipDesign>` with `fleet: Vec<ShipDesign>` in `core/src/resource/mod.rs`
- [x] 1.4 Rename `load_current_ship` to `load_default_fleet` in `core/src/system/mod.rs` and update its registration in `core/src/main.rs`

## 2. Fleet persistence module

- [x] 2.1 Create `core/src/save/fleet/` with `mod.rs`, `component.rs`, `lib.rs`, `system.rs`, registered in the save and app plugin wiring
- [x] 2.2 Define a versioned `RonDocument` for the fleet written atomically to `<data_dir>/fleet.ron` using `save::document` helpers
- [x] 2.3 Implement load-or-seed: load the persisted fleet, otherwise resolve `default_fleet` names against `ShipCatalog`, warning and skipping unknown names
- [x] 2.4 Restore config defaults with a warning when the loaded fleet is empty
- [x] 2.5 Rewrite `fleet.ron` atomically on deploy/remove; warn and keep the in-memory fleet on write failure
- [x] 2.6 Add tests for fleet RON round-trip, persisted-fleet precedence over config, empty-fleet reseed, and unknown-name skip

## 3. Mission schema and spawning

- [x] 3.1 Remove `player: ShipSpawn` from `MissionDef` and add `spawn: Vec2` in `core/src/mission/def.rs`
- [x] 3.2 Update `core/assets/config/mission/quick-battle.ron`: remove `player`, add `spawn: (5000.0, 0.0)`, leave enemies unchanged
- [x] 3.3 Change `spawn_mission` to accept `&[ShipDesign]` and the spacing value, spawning fleet ships via `spawn_ship_entity` with `DEFAULT_TEAM_ID`
- [x] 3.4 Add a pure line-formation helper centered on the mission anchor and unit-test its positions
- [x] 3.5 Update `poll_mission_loaded` to pass `&state.fleet` and `config.0.fleet_spacing`; keep `ShipCatalog` for enemy schematic lookup
- [x] 3.6 Keep `find_schematic` fallback behavior unchanged for enemies and confirm via existing mission tests

## 4. Editor deploy and fleet management

- [x] 4.1 Change `on_deploy` to upsert the grid design into the fleet by name and persist the change
- [x] 4.2 Change `sync_deploy_button` to compare name, doctrine, and block position/type/rotation while ignoring `BlockPlacement.id`
- [x] 4.3 Add a deployed-fleet list to the editor UI with one remove action per ship
- [x] 4.4 Wire the remove action to update `DeployPayload.fleet` and persist, without deleting the design from `ShipCatalog`
- [x] 4.5 Add tests for upsert-by-name, unchanged comparison, and remove behavior

## 5. Docs and verification

- [x] 5.1 Rewrite the player/deploy sections of `docs/mission.md` as spec for the fleet model, config defaults, persistence, and formation spawn
- [x] 5.2 Add or update spec coverage in `docs/` for any editor fleet UI behavior not already described
- [x] 5.3 Run `cargo check` and `cargo clippy` scoped to the touched crates, then `cargo test` for the new and affected tests
- [ ] 5.4 Run `cargo dev` in the background and verify Quick Battle spawns `Interceptor` and `B2` as team-0 ships at the mission anchor, then terminate it
