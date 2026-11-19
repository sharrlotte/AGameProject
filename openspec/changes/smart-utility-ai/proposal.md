## Why

The current AI system relies on manually authored `AiProfile` rule lists inside every ship `.ron` file, ignores live ship state (surviving blocks, weapon rotation vs. fixed mounts, weapon archetypes, power/battery starvation, ammo depletion, support blocks, and radar range), uses an omniscient `ShipIndex`, restricts AI flight to forward-only thrust, and has no scalable difficulty or teamwork. Replacing per-ship `.ron` rules with live block-introspective Utility AI, a 1–10 `Skill` curve, always-on team-wide intel sharing, and player-parity decoupled steering makes ships automatically fight to their actual build and scale intelligently with difficulty.

## What Changes

- **BREAKING**: Remove per-ship `AiProfile`, `AiRule`, `AiCondition`, `BehaviorSpec`, `TargetRef`, `PositionRef`, `AiBehavior`, `AiRuntime`, `PRESETS`, and `ShipBehavior`.
- **BREAKING**: Bump `DESIGN_SCHEMA_VERSION` from `2` to `3` (`ShipDesign` replaces `ai: AiProfile` with `#[serde(default)] pub doctrine: Doctrine`) and update all 4 shipped starter `.ron` files (`interceptor`, `raider`, `b2`, `blackbird`).
- **BREAKING**: Bump `SAVE_SCHEMA_VERSION` from `6` to `7` (`ShipSnapshot` persists `doctrine`, `skill`, and `BrainSave`; `WorldSave` persists team intel state).
- Add pure `Feature`-driven live ship introspection (`Capability`) recomputed on spawn and `ShipChanged` (fixed vs. rotating turrets & `Rot90` mount angles, weapon archetypes, DPS-weighted ranges, power/ammo dependency, `Booster`/support blocks, 4-quadrant armor HP, live `max_radar_range`, and inferred `Role`).
- Add continuous `Skill(1..=10)` difficulty resolved via `ShipSpawn -> MissionDef -> GameSetting -> default_enemy_level` (and `ShipSpawn -> default_ally_level` for player-team ships), backed by a global hot-reloadable `core/assets/config/base/ai.ron` tuning table.
- Add live `max_radar_range` perception (`Sensor`) feeding an always-on team-wide `TeamIntel` bus (shared radar sightings, shared `DamageEvent` hit alerts, and decaying team memory even if the spotter loses contact or dies) plus 2-tier `TacticalBoard` teamwork (shared focus-fire target and `Support`/`Escort` VIP pairing at mid+ `Skill`).
- Add a modular Utility AI decision engine (`Brain` + composable `Scorer`s with hysteresis) supporting `Engage`, `Recharge`, `Retreat`, `Support`, `Regroup`, `Investigate`, and `Search`, plus `Skill`-gated `Booster` power-shedding (`Disabled`) and throttle budgeting (loot drops remain player-only).
- Replace `ShipBehavior` with decoupled `Steering { desired_heading, thrust_world, throttle_cap }` (omnidirectional thrust progressively unlocked by `Skill`, spinal lead aiming, spinal-line dodging, and 4-quadrant armor angling).
- Add ship `FireOrder { primary_target, lead_accuracy }` consumed by combat turrets (`Closest`/`MostHP`), leaving point-defense (`EnemyBullet`) autonomous, and fix `turret_tracking` to aim at `state.aim_point`.
- Update Settings UI with an `AI Skill (1..10)` stepper, replace the Editor AI tab with `Doctrine` selector buttons, and update dev ship debug text/gizmos for `Brain`, `Skill`, `Doctrine`, `Capability`, and `Steering`.
- Update `/docs/ai.md`, `/docs/config.md`, `/docs/save.md`, and `/docs/ui.md` specs.

## Capabilities

### New Capabilities
- `smart-utility-ai`: Block-introspective `Capability` analysis, `Skill(1..=10)` curve & unlock table in `base/ai.ron`, modular Utility AI (`Brain` + `Scorer`s with hysteresis), power/ammo/support block tactics (throttle budgeting, `Recharge`, `Booster` power-shedding), and decoupled `Steering`.
- `ai-team-intel`: Live `max_radar_range` scanning (`Sensor`), always-on team-wide `TeamIntel` bus (shared radar sightings, shared `DamageEvent` alerts, decaying team memory), and 2-tier `TacticalBoard` teamwork (focus fire and VIP `Support`/`Escort` pairing).

### Modified Capabilities
- `ai-profiles`: Replace inline per-ship `.ron` `AiProfile` rules and `AiBehavior` / `AiRuntime` with `Doctrine` (`Auto`, `Brawler`, `Skirmisher`, `Sniper`, `Support`, `Escort`), `Skill`, `Capability`, `Sensor`, and `Brain`.
- `fixed-spinal-aim`: Supersede `ShipBehavior::AimAt` with decoupled `Steering` supporting fixed weapons at any `Rot90` alongside omnidirectional thrust; coordinate combat turret focus-fire and `Skill`-scaled aim lead via `FireOrder` and track `state.aim_point` in `turret_tracking`.
- `editor-ui`: Replace Editor AI rule/preset RON preview with `Doctrine` selector buttons and add `AI Skill (1..10)` stepper to Settings UI.
- `world-save`: Bump `DESIGN_SCHEMA_VERSION = 3` and `SAVE_SCHEMA_VERSION = 7` to persist `Doctrine`, `Skill`, `BrainSave`, and `TeamIntel` cleanly.
- `ship-debug`: Update dev debug text and gizmos to inspect `Brain`, `Skill`, `Doctrine`, `Capability`, and `Steering` instead of removed `AiRuntime` and `ShipBehavior`.

## Impact

- **Core AI**: `core/src/content/ship/ai/` (rewritten into modular `mod.rs`, `component.rs`, `config.rs`, `capability.rs`, `sensor.rs`, `board.rs`, `scorer.rs`, `steer.rs`) and `core/assets/config/base/ai.ron`.
- **Ship Physics & Turrets**: `core/src/content/ship/{component.rs, system.rs, mod.rs}` (`Steering`, `FireOrder`, `update_ship_steering`) and `core/src/content/block/feature/turret/system.rs` (`FireOrder` integration and `turret_tracking` lead fix).
- **Config & Missions**: `core/src/config/{mod.rs, setting.rs, loader.rs}`, `core/src/mission/{def.rs, mod.rs}`, `core/assets/config/setting.ron`, `core/assets/config/starter/*.ron`.
- **Save & UI**: `core/src/save/{design/, battle/}`, `core/src/scene/{editor/, setting/}`, and dev ship debug systems.
- **Docs**: `docs/{ai.md, config.md, save.md, ui.md}`.
