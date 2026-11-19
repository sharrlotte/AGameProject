## Why

Missions currently hard-code a single player slot (`MissionDef.player: ShipSpawn`), and `DeployPayload.current_ship: Option<ShipDesign>` can hold only one deployed design. The engine already treats team 0 as a plural set (HUD ship list, control switching, outcome counting, save/load), so the schema is the only thing preventing a player fleet. This change makes the deployed fleet the single source of truth for player ships, persists it across restarts, and reduces missions to battlefield definitions.

## What Changes

- **BREAKING** Remove `player: ShipSpawn` from `MissionDef` and from `assets/config/mission/quick-battle.ron`; add `spawn: Vec2` as the fleet's spawn anchor. Enemy spawns are unchanged.
- Replace `DeployPayload.current_ship: Option<ShipDesign>` with `DeployPayload.fleet: Vec<ShipDesign>`. All fleet ships are forced to team 0.
- Add `default_fleet: Vec<String>` (`["Interceptor", "B2"]`) and `fleet_spacing: f32` (`400.0`) to `GameConfig` / `config/game.ron`. On first run the fleet is seeded by resolving those names against `ShipCatalog`; unresolved names are warned and skipped.
- Persist the fleet as a versioned RON document at `<data_dir>/fleet.ron`. A saved fleet replaces the config defaults (no merge). If the loaded fleet is empty, restore the config defaults with a warning so a battle can never start with zero player ships.
- Editor Deploy now upserts the current grid design into the fleet by name instead of replacing a single slot; the editor shows the deployed fleet with a remove action per ship. Upsert writes happen immediately.
- `spawn_mission` spawns the fleet at the mission anchor in a line formation (`fleet_spacing`), and enemies by schematic (`find_schematic` fallback retained for enemies only).
- Rename `load_current_ship` to `load_default_fleet`.
- Record the fleet model and deploy behavior in `docs/mission.md` as spec.

## Capabilities

### New Capabilities
- `deploy-fleet`: Persisted, user-managed player fleet that is the sole source of truth for team-0 ships, seeded from config, spawned at a mission anchor, and validated against an empty-fleet soft-lock.

### Modified Capabilities
- `editor-ui`: Deploy upserts a design into the fleet (by name) instead of replacing a single ship; a fleet list with per-ship removal is exposed.
- `starter-roles`: Quick Battle's player roster is the config-driven default fleet (`Interceptor` + `B2`) rather than a mission-declared single `Interceptor`; enemy roster stays mission-defined.
- `config-validation`: Mission configs no longer declare player teams; game config validates the new fleet fields (`default_fleet` names resolve against the catalog, `fleet_spacing` is non-negative).

## Impact

- `core/src/resource/mod.rs`, `core/src/system/mod.rs`.
- `core/src/config/game.rs`, `core/assets/config/base/game.ron`.
- `core/src/mission/{def,state,mod}.rs`, `core/assets/config/mission/quick-battle.ron`.
- New `core/src/save/fleet/` module (`mod.rs`, `component.rs`, `lib.rs`, `system.rs`) reusing `save::document::RonDocument`.
- `core/src/scene/editor/{input,ui}.rs` and the editor plugin registration in `core/src/scene/editor/mod.rs`.
- `core/src/main.rs` (startup system rename).
- Existing specs `editor-ui`, `starter-roles`, `config-validation`; `docs/mission.md`.
