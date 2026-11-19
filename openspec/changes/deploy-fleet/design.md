## Context

`MissionDef` currently declares one `player: ShipSpawn` and `DeployPayload` holds one `current_ship: Option<ShipDesign>`. `load_current_ship` fills that slot at startup from the catalog's first design, so in practice the mission's `player.schematic_name` is ignored and only one player ship can ever spawn. Everything downstream already handles many team-0 ships: the HUD ship list, `F` control switching, outcome attrition, team collision groups, and world save/load. The mismatch is purely in the data model and spawn path.

Constraints from the project rules: SOLID/small modules, no `unwrap` or hardcoded values in production code, inputs validated, data in RON with hot-reload-friendly config, events for communication, single source of truth, no dev-only code in production, and changes recorded in `/docs` as spec only.

## Goals / Non-Goals

**Goals:**

- Make a persisted `DeployPayload.fleet: Vec<ShipDesign>` the only source of truth for team-0 ships.
- Seed a first-run fleet from `game.ron` names resolved against `ShipCatalog`.
- Persist the fleet across restarts and let the player manage it in the editor (upsert + remove).
- Spawn the fleet at a mission-provided anchor in a configurable line formation.
- Keep enemy spawning and battle outcome logic unchanged.
- Update `docs/mission.md` as spec.

**Non-Goals:**

- Other mission objectives, mission select, campaign map, or rewards.
- Changing enemy schematic resolution: `find_schematic` keeps its silent first-design fallback (known defect in `docs/defects.md`).
- Storing the fleet in the campaign save or world snapshot.
- A fleet cap via the currently unused `max_ships` config value.
- Grid/circular formations or per-design spawn positions.
- Persisting editor-selected designs into the catalog when removed from the fleet (removal changes membership only).

## Decisions

### 1. Fleet replaces the mission player slot

`MissionDef.player` is removed; `MissionDef.spawn: Vec2` becomes the anchor. Alternatives considered: deploy-as-flagship (slot 0), uniform fleet (all slots use the deployed design), append-to-mission-roster, and ignore-deploy. All were rejected because they keep a second source of truth in the mission file, and because `load_current_ship` always populates the deploy slot, meaning a mission-declared roster would either be ignored or duplicated. Removing the field makes the ownership unambiguous.

### 2. `DeployPayload.fleet: Vec<ShipDesign>`, team forced to 0

Each fleet entry is a full design snapshot. All spawned fleet ships use `Team::new(DEFAULT_TEAM_ID)` regardless of any team in mission data, matching `Team::is_player()` and the existing collision/attrition semantics. Honoring a per-entry team was rejected: a team-1 "player" would fight the fleet and break friendly-fire grouping. The `ShipSpawn.team` field remains for enemies only.

### 3. Config-driven defaults

`GameConfig` gains `#[serde(default)] default_fleet: Vec<String>` = `["Interceptor", "B2"]` and `fleet_spacing: f32` = `400.0`. `load_default_fleet` resolves names against `ShipCatalog`, warns and skips unknown names, and uses the config list only when no persisted fleet exists. Alternatives: all catalog designs (mixes user saves into defaults), hardcoded starter constants (violates the no-hardcode rule), and `Interceptor` only (not a multi-ship fleet). A config list keeps names in RON and allows future changes without code edits.

### 4. Mission anchor + pure line formation

`MissionDef.spawn` supplies the anchor; `fleet_spacing` comes from `game.ron`. A pure helper computes `anchor + Vec2::Y * (i - (n-1)/2) * spacing`, making positions testable without a world. Alternatives: config-only anchor (loses per-mission placement), a `spawn` field on `ShipDesign` (mixes deployment into design data), and a grid formation (deferred; the line helper can be extended later).

### 5. Upsert by name in the editor

Deploy builds a fresh `ShipDesign` with a new UUID and upserts by `name`: replace the existing entry, or append if the name is new. The Deploy button is disabled only while the current editor design matches the fleet entry. That comparison MUST ignore `BlockPlacement.id` (regenerated when blocks are placed) and compare `name`, `doctrine`, and `(position, block_type, rotation)` triples. Alternatives: strict dedupe (edited designs never update) and always-append (unlimited duplicates, no disable state). Upsert is what makes "edit then redeploy" work.

### 6. Versioned RON persistence with replace semantics

A new `save/fleet/` module defines a `RonDocument` for the fleet, written atomically to `<data_dir>/fleet.ron` using the existing `save::document` helpers. A saved fleet is authoritative; config defaults do not merge into it. If the loaded fleet is empty, defaults are restored with a warning so a battle cannot start with zero player ships. Alternatives: session-only (loses customization), campaign save (unnecessary coupling), and merge-by-name (resurrects deliberately removed ships). Given the fleet stores full design snapshots, a catalog design edited on disk is not reflected until the player redeploys it — that is the accepted cost of snapshot-based membership.

### 7. Editor fleet list with removal

The editor shows the deployed fleet with one remove action per ship. Removal mutates only `DeployPayload.fleet` and rewrites `fleet.ron`; it never deletes the design from `ShipCatalog`. This is required to undo a mistaken deployment and to make an empty fleet recoverable via auto-reseed.

### 8. Split spawn paths

`spawn_mission` takes `&[ShipDesign]` and spawns fleet ships directly with `spawn_ship_entity`; enemies still go through `spawn_ship` / `find_schematic`. This keeps design-based and schematic-based spawning separate and removes `ShipCatalog` from the mission system parameters. `battle-outcome` needs no change because it already counts all team-0 ships.

### 9. Validation

`GameConfig::validate` requires `fleet_spacing >= 0`. Unresolved `default_fleet` names are warned and skipped rather than aborting startup, because a stale config name should not prevent the game from loading. An empty fleet after loading is never accepted (auto-reseed). Mission configs no longer carry player teams, so validation of "valid player teams" is dropped from mission requirements.

## Risks / Trade-offs

- [A saved fleet never picks up newly configured defaults] → Documented; players add ships by deploying designs in the editor. Deleting `fleet.ron` restores config defaults.
- [Fleet designs are snapshots, not references] → Editing a catalog design does not update an already-deployed ship; redeploy to replace it.
- [Line formation can overlap for very large hulls] → `fleet_spacing` is a RON value; no hardcoded geometry.
- [Persistence write failure] → Reuse atomic document writes and warn; the in-memory fleet stays usable for the session.
- [Missing enemy schematic still silently spawns the first design] → Pre-existing defect, explicitly out of scope; remains tracked in `docs/defects.md`.
- [Unbounded fleet size] → `max_ships` is unused and stays untouched; the editor list is the practical limit.
- [Breaking mission data format] → Only one mission RON exists in the repo and it is updated in the same change; no migration tooling needed.

## Migration Plan

1. Update `game.ron` with `default_fleet` and `fleet_spacing`; update `quick-battle.ron` by removing `player` and adding `spawn`.
2. On first launch, no `fleet.ron` exists, so `load_default_fleet` seeds `Interceptor` + `B2` from the catalog and writes the document on the first deploy/remove.
3. Previously deployed single ships are not migrated; defaults replace them. Rollback is achieved by reverting the change and deleting `fleet.ron`.

## Open Questions

None. All design decisions were resolved before this proposal was created.
