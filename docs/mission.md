# Mission Spec — Battle-Core Scaffolding

## Scope (v1)

One battle. Victory = all enemy ships destroyed; defeat = all player ships destroyed
("DestroyEnemyCore" objective — with Core-equipped ships, the two are the same). Other
objectives (`Survive`, `Collect`, `Escort`, waves) are reserved in the schema and marked
future work; they are not implemented.

`BattlePhase` is a Bevy state: `Idle → Loading → Active → Ended`. `Idle` is the default
resting state; `SceneState::Game` entry moves it to `Loading` (mission asset kicks off
async load). When the mission def is read, ships spawn and phase becomes `Active`. Core
destruction emits `ShipDestroyed { ship, team }` from the ship mutation pipeline; the
mission system consumes that message and queries remaining ships by team to decide the
outcome. Leaving Game resets the phase to `Idle` and clears `BattleResult`. On `Ended`,
the overlay spawns.

## Player fleet is the source of truth

Missions do not declare player ships. `MissionDef` contains `spawn: Vec2` (the fleet
anchor), `enemies: Vec<ShipSpawn>`, `objective`, and optional metadata. Every team-0
ship is spawned from `DeployPayload.fleet: Vec<ShipDesign>`, forced to
`Team::new(DEFAULT_TEAM_ID)`. `ShipSpawn.team` therefore applies to enemies only.

A fresh install seeds the fleet by resolving `default_fleet` names from
`config/base/game.ron` against the active catalog. The configured defaults on a fresh
install are `["Interceptor", "B2"]`. Unresolved names are warned and skipped.

Entering a save slot hydrates `ShipCatalog` from that slot's `designs` table and seeds
`DeployPayload.fleet` from it; Home and `QuickBattle` use the `<data_dir>/sandbox_designs/`
catalog. Fleet membership is therefore per-slot and in memory — there is no global
`fleet.ron`. Removing a ship changes fleet membership only and never deletes its design.
Because the fleet stores full design snapshots, an edit to a catalog design is reflected
only after redeploying that design.

Fleet ships spawn on a line centered on `MissionDef.spawn`. Each design's hull extent is
measured from its block layout via `BlockRegistry` and `BlockGrid`, and ships are placed
edge-to-edge with `fleet_spacing` from `game.ron` (default `400.0`) as the **gap between
hulls**, never between centers. Asymmetric hulls are handled by using each design's own
minimum/maximum cell offsets relative to its origin. This prevents large designs such as
`B2` from overlapping smaller ones at spawn. The offset math is a pure function of the
anchor, hull extents, and gap; only the extent lookup needs the registry.

## Quick Battle flow

Home → **Play** → Save Slot Picker → `New World` / `Load` (`BattleMode::Campaign`), or
Home → **Quick Battle** (`StartQuickBattleEvent`, `BattleMode::QuickBattle`) → `SceneState::Game`:

1. `setup` spawns the HUD; `MissionPlugin::setup_battle` inserts `MissionAsset` (loads
   `assets/config/mission/quick-battle.ron`) and resets `BattlePhase` to `Loading`.
2. `poll_mission_loaded` spawns the mission when the asset read finishes (~1 frame; the
   asset is cached afterward).
3. Player ships = `DeployPayload.fleet`, each at a bounds-aware line position derived
   from `MissionDef.spawn` and `fleet_spacing`, team 0. Enemies are resolved by
   `schematic_name` from their absolute positions in the mission, team 1.
   (`find_schematic` falls back to the first available design when a name is missing —
   see defects.md.)
4. Core loss in `apply_body_mutations` emits `ShipDestroyed` and wrecks the surviving
   components as debris bodies.
5. The outcome system consumes `ShipDestroyed` in `FixedPostUpdate` after the mutation
   pipeline, queries remaining ships by team, and sets
   `BattleResult { started_at, outcome, duration_secs }`. `OnEnter(BattlePhase::Ended)`
   spawns the overlay.
6. Overlay buttons: **Retry** → `BattleRestart(true)` + Home → Home's
   `resume_battle_on_restart` sees the flag, clears it, and re-enters Game fresh (full
   respawn, mission reload). **Home** → Home, battle stays over until next entry.
7. Re-entering Game resets `BattlePhase` to `Loading` in `setup_battle` — mission always
   re-spawns clean.
8. `DespawnOnExit(SceneState::Game)` cleanup removes all battle entities on scene exit,
   as everywhere else.

## Data files

`assets/config/mission/quick-battle.ron` — the only mission for now:

```ron
(
    name: "Quick Battle",
    briefing: "Two Hive lances hold the frontier. Break their Cores.",
    spawn: (5000.0, 0.0),
    enemies: [
        (team: 1, position: (-3000.0, 0.0), schematic_name: "Blackbird"),
        (team: 1, position: (0.0, 3000.0), schematic_name: "Raider"),
        (team: 1, position: (0.0, -3000.0), schematic_name: "Interceptor"),
    ],
    objective: DestroyEnemyCore,
)
```

`MissionDef` — derives `Asset`, `TypePath`, `Deserialize`, `Clone`, `PartialEq`, `Debug`
and uses `deny_unknown_fields`. It mirrors `ShipSpawn { team: u8, position: Vec2,
schematic_name: String, ai_level: Option<u8> }` for enemies only and `MissionObjective`
(only `DestroyEnemyCore` valid today). A `player` field is rejected by the schema.

`assets/config/base/game.ron` adds `default_fleet: ["Interceptor", "B2"]` and
`fleet_spacing: 400.0`. `fleet_spacing` is validated as non-negative by `GameConfig`.

## Guarantees and invariants

- Mission spawn path is the **only** ship spawn path in Game — the old top-up spawner
  (`check_ship`) is deleted; `MAX_FACTIONS` deleted; `SchematicCache`,
  `mark_schematics_dirty` deleted; `utils::random_vec2_step` deleted.
- `DeployPayload.fleet` is the only source of player ships; every fleet ship is team 0.
- Mission configs never carry a player slot or player team.
- Fleet ownership is scoped to the active save slot (or Home sandbox) and seeded from the
  catalog on session entry; config defaults apply only when a name cannot be resolved.
- Outcome detection is event-driven: `ShipDestroyed` is emitted only when a ship's core
  is lost; scene teardown does not emit it. Counting uses only `Ship` entities —
  blocks/bullets/lasers/missiles don't count toward attrition.
- Outcome detection, mission loading, and overlay buttons run only in `SceneState::Game`.
- `BattlePhase` is a `States`; `Idle` is the default; exiting Game resets the phase and
  `BattleResult`. `Ended` overlay and buttons are `DespawnOnExit(SceneState::Game)` — they
  despawn on scene exit.
- `handle_pause_input` is unchanged — Escape toggles pause during battle as before
  (the overlay does not block the pause menu; deliberate: battle stays accessible).

## Explicitly not done (future)

- Other objectives (survive/collect/escort/waves), mission select screen, campaign map.
- Difficulty scaling and mission-specific AI profiles beyond the starter designs.
- Grid/circular fleet formations or per-design spawn positions.
- A fleet size cap via `max_ships`.
- Victory rewards/meta progression of any kind.
