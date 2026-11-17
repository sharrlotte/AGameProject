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

## Quick Battle flow

Home → **Quick Battle** (or **Play** — same battle in v1) → `SceneState::Game`:

1. `setup` spawns the HUD; `MissionPlugin::setup_battle` inserts `MissionAsset` (loads
   `assets/config/mission/quick-battle.ron`) and resets `BattlePhase` to `Loading`.
2. `poll_mission_loaded` spawns the mission when the asset read finishes (~1 frame; the
   asset is cached afterward).
3. Player ship = `DeployPayload.current_ship` if the editor Deployed one, otherwise the
   `ship_0` schematic (a fresh Core + LazerTurret hull). Enemies = `ship_1`, `ship_2` from
   their absolute spawn positions, team 1.
4. Core loss in `apply_ship_mutations` emits `ShipDestroyed` and marks the ship `Death`.
5. The outcome system consumes `ShipDestroyed` in `FixedPostUpdate` after the mutation
   pipeline, queries remaining ships by team, and sets
   `BattleResult { started_at, outcome, duration_secs }`. `OnEnter(BattlePhase::Ended)`
   spawns the overlay.
6. Overlay buttons: **Retry** → `BattleRestart(true)` + Home → Home's
   `resume_battle_on_restart` sees the flag, clears it, and re-enters Game fresh (full
   respawn, mission reload). **Home** → Home, battle stays over until next entry.
7. Re-entering Game resets `BattlePhase` to `Loading` in `setup_battle` — mission always
   re-spawns clean.
8. `SceneEntity(Game)` cleanup removes all battle entities on scene exit, as everywhere
   else.

## Data files

`assets/config/mission/quick-battle.ron` — the only mission for now:

```ron
(
    name: "Quick Battle",
    briefing: "Two Hive lances hold the frontier. Break their Cores.",
    player: (team: 0, position: (0.0, 0.0), schematic_name: "ship_0"),
    enemies: [
        (team: 1, position: (6000.0, 0.0), schematic_name: "ship_1"),
        (team: 1, position: (-6000.0, 0.0), schematic_name: "ship_2"),
    ],
    objective: DestroyEnemyCore,
)
```

`MissionDef` — derives `Asset`, `TypePath`, `Deserialize`, `deny_unknown_fields`; mirrors
`ShipSpawn { team: u8, position: Vec2, schematic_name: String }` and
`MissionObjective` (only `DestroyEnemyCore` valid today).

Schematic name lookup: `find_schematic` resolves by `name` (placeholders are
`ship_0..ship_4` on fresh installs); `schematic.to_ship_data()` generates fresh UUIDs
as usual. Empty/missing schematics spawn nothing and log a warning.

## Guarantees and invariants

- Mission spawn path is the **only** ship spawn path in Game — the old top-up spawner
  (`check_ship`) is deleted; `MAX_FACTIONS` deleted; `SchematicCache`, `mark_schematics_dirty`
  deleted; `utils::random_vec2_step` deleted.
- Player fallback order: Deploy → `ship_0` → (missing) first available schematic.
- The mission asset is cached after first read; Retry/Home re-entry re-uses it.
- Outcome detection is event-driven: `ShipDestroyed` is emitted only when a ship's core
  is lost; scene teardown does not emit it. Counting uses only `Ship` entities —
  blocks/bullets/lasers/missiles don't count toward attrition.
- Outcome detection, mission loading, and overlay buttons run only in `SceneState::Game`.
- `BattlePhase` is a `States`; `Idle` is the default; exiting Game resets the phase and
  `BattleResult`. `Ended` overlay and buttons are `SceneEntity(Game)` — they despawn on
  scene exit.
- `handle_pause_input` is unchanged — Escape toggles pause during battle as before
  (the overlay does not block the pause menu; deliberate: battle stays accessible).

## Explicitly not done (future)

- Other objectives (survive/collect/escort/waves), mission select screen, campaign map.
- Difficulty scaling and mission-specific AI profiles beyond the starter designs.
- `quick-battle.ron` is the only mission; no mission hot reload pipeline beyond the
  existing loader (it reloads only on scene re-entry).
- Victory rewards/meta progression of any kind.
