## Context

The current AI system in `core/src/content/ship/ai/` uses per-ship inline `AiProfile { initial, rules }` serialized inside every `ShipDesign` `.ron` file. This architecture has several fundamental limitations:
- Designers must author hardcoded distances and HP thresholds per ship in `.ron`, which go stale when blocks are swapped or destroyed mid-battle.
- Several live bugs exist in the evaluator (`EnemyInRange` is negated, `HpBelow`/`HpAbove` compare raw HP against `0.0..1.0` ratios, and `Kite` never resolves a target).
- `ShipIndex` gives every AI omniscient map-wide awareness, ignoring block/turret radar ranges.
- `ShipBehavior` restricts AI to forward-only thrust (`Approach`, `GoAround`, `AimAt`), whereas player ships (`player_control_ship`) already support full omnidirectional thrust (`W/S` + `Q/E`) independent of hull rotation (`A/D`).
- `turret_tracking` aims barrels at raw `ship.core_position` instead of the computed `state.aim_point` lead point, and turrets have no ship-level focus-fire or difficulty-scaled lead accuracy.
- There is no difficulty scaling (`1..=10`), no power/ammo/support block awareness, and no team-wide intel sharing or coordination.

## Goals / Non-Goals

**Goals:**
- Eliminate per-ship AI rule authoring in `.ron`; replace with live pure-`Feature` block introspection (`Capability`) plus an optional `Doctrine` override (`Auto`, `Brawler`, `Skirmisher`, `Sniper`, `Support`, `Escort`).
- Support continuous numeric `Skill(1..=10)` difficulty backed by one global hot-reloadable `core/assets/config/base/ai.ron` curve and unlock table, resolved via `ShipSpawn -> MissionDef -> GameSetting -> default_enemy_level` (and `ShipSpawn -> default_ally_level` for player-team ships).
- Implement true `max_radar_range` scanning (`Sensor`) feeding an always-on team-wide `TeamIntel` bus (shared radar sightings, shared `DamageEvent` hit alerts, and decaying team memory even if the spotter loses contact or dies).
- Implement 2-tier teamwork via `TacticalBoard` / `TeamPlan`: always-on `TeamIntel` across all skill levels, plus mid+ `Skill` shared focus-fire and VIP `Support`/`Escort` pairing (`Booster` overdrive aura / point-defense screening).
- Implement a modular Utility AI (`Brain` + composable `Scorer`s with hysteresis) covering `Engage`, `Recharge`, `Retreat`, `Support`, `Regroup`, `Investigate`, and `Search`, plus `Skill`-gated `Booster` power-shedding (`Disabled`) and throttle budgeting.
- Replace `ShipBehavior` with decoupled `Steering { desired_heading, thrust_world, throttle_cap }` using player-parity omnidirectional thrust (progressively unlocked by `Skill`), spinal lead aiming at any `Rot90`, spinal-line dodging, and 4-quadrant armor angling.
- Coordinate combat turret focus-fire and `Skill`-scaled aim lead via `FireOrder` (keeping `EnemyBullet` point-defense autonomous) and fix `turret_tracking` to track `state.aim_point`.
- Provide minimal UI updates (Settings `AI Skill 1..10` stepper; Editor AI tab `Doctrine` selector buttons), bump schemas (`DESIGN_SCHEMA_VERSION = 3`, `SAVE_SCHEMA_VERSION = 7`), update dev ship debug inspection, and update `/docs`.

**Non-Goals:**
- Enemy AI scavenging `steel` drops (loot drops remain strictly player-only).
- Live capability readout text in the Editor AI tab (kept minimal: `Doctrine` buttons only).
- Flanking angular slot assignment or tractor-beam weapon combos (explicitly scoped to 2-tier teamwork: shared `TeamIntel` + focus fire & support pairing).
- Changing player manual flight controls (`ControlMode::Pilot`).

## Decisions

### 1. Ship Configuration: `Doctrine` Replacing `AiProfile`
- **Choice**: `ShipDesign` stores `#[serde(default)] pub doctrine: Doctrine` where `Doctrine` is `Auto | Brawler | Skirmisher | Sniper | Support | Escort`. All `AiProfile` / `AiRule` / `AiCondition` / `BehaviorSpec` / `TargetRef` / `PositionRef` / `PRESETS` types are removed.
- **Rationale**: Ships adapt dynamically to their surviving blocks without stale `.ron` rules, while still letting a designer mark an identical hull as `Escort` or `Support` when desired.
- **Alternatives considered**: Pure auto with zero fields (cannot distinguish identical hulls assigned different fleet roles); compact personality knobs (`aggression`, `caution`) (unnecessary per-ship tuning burden).

### 2. Pure `Feature` Introspection (`Capability`)
- **Choice**: Derive `Capability` exclusively from `BlockDef.features` and live `BlockGrid` placements (recomputed on spawn and `ShipChanged`). Zero hardcoded block string IDs and zero duplicate `ai_tags` in `block.ron`.
- **Details**:
  - `Feature::Turret(t)`: classifies fixed (`t.turn_speed == 0.0`, recording `Rot90` mount angle and `aim_tolerance`) vs. rotating (`t.turn_speed > 0.0`), `ShootPattern::LaserBeam` (sustained drain), `ShootPattern::PullingBeam` (tractor), `TargetingStrategy::EnemyBullet` (point defense), projectile speed, DPS estimate, `TurretPower` (`Charge` vs. `Drain`), and `Ammo` dependency.
  - `Feature::Booster(b)`: records overdrive count, max `radius`, and total `rate`.
  - `Feature::Generator` / `Feature::Battery` / `Feature::Thruster`: records mobility (`thrust_to_mass`, `can_move`) and power infrastructure.
  - 4-quadrant armor HP (`[Front, Right, Back, Left]` relative to core) computed from surviving block positions & health for high-skill armor angling.
  - `max_radar_range`: max of `base_core_radar_range` (from `base/ai.ron`) and all surviving turret/sensor ranges.
  - `Role` inference: `Doctrine::Auto` compares `offensive_dps` against `support_weight` (auto-detecting dedicated support tenders carrying `Booster`/`PullingBeam`/`EnemyBullet` PD with weak/no main guns), then classifies `Sniper` (high fixed-DPS ratio & long range), `Skirmisher` (mid-long range & high thrust-to-mass), or `Brawler`.
- **Rationale**: Single source of truth (`AGENTS.md`); impossible for AI classification to drift from actual block mechanics.

### 3. Continuous `Skill(1..=10)` & Hot-Reloadable `core/assets/config/base/ai.ron`
- **Choice**: `Skill(pub u8)` (clamped `1..=10`) interpolates continuous curves (`t = (level - 1) as f32 / 9.0`) and checks integer unlock thresholds loaded from `core/assets/config/base/ai.ron` (`AiConfig` / `AiConfigValue`, validated on load).
- **Resolution order at spawn**:
  1. `ShipSpawn.ai_level` (explicit per-spawn override)
  2. If `team.is_player()` -> `ai_cfg.default_ally_level` (default `8`)
  3. Else `MissionDef.ai_level` -> `GameSetting.ai_level` -> `ai_cfg.default_enemy_level` (default `5`)
- **Curves & unlocks**: `think_rate_hz`, `lead_accuracy`, `aim_jitter_rad`, `range_precision`, `evasion_strength`, `memory_secs`; unlocks for `omnidirectional_thrust_level`, `power_management_level`, `focus_fire_level`, `strafe_evade_level`, `spinal_dodge_level`, `armor_angling_level`, and `support_coordination_level`.
- **Rationale**: Keeps all balance constants data-driven and hot-reloadable without requiring per-ship AI rule authoring.

### 4. Always-On `TeamIntel` & 2-Tier `TacticalBoard`
- **Choice**: Replace `ShipIndex` with `TacticalBoard { teams: HashMap<u8, TeamPlan> }`.
- **Tier 1 (Always On, `Skill 1..=10`) — `TeamIntel`**:
  - Each ship scans enemy ships within its own `Capability.max_radar_range`.
  - Any sighting immediately inserts/refreshes a `SharedContact` in its team's `TeamIntel.contacts`, visible to every ship on the same team.
  - Any `DamageEvent` from an attacker outside radar range posts a shared `TeamAlert` (origin direction / estimated position, decaying TTL) to `TeamIntel.alerts`.
  - When a live contact is no longer spotted (including if the spotter is destroyed), it moves into `TeamIntel.last_known` for `memory_secs` before expiring.
- **Tier 2 (Mid+ `Skill`) — `TeamPlan`**:
  - Computes `focus_target` (prioritizing exposed/low-HP or highest-threat enemy in `TeamIntel.contacts`) for ships with `skill >= focus_fire_level`.
  - Computes `vip_ally` (highest-DPS allied combat ship) and `rally_point` (allied centroid) for `Support`/`Escort` pairing (`skill >= support_coordination_level` or explicit `Doctrine::{Support, Escort}`).
- **Rationale**: Satisfies fleet-wide threat alerting at all difficulties while keeping `Skill` progression clear and cleanly scoped.

### 5. Modular Utility AI (`Brain` + `Scorer`s + Hysteresis) & Power Shedding
- **Choice**: `Brain` stores `current: Tactic`, `spawn_position: Vec2`, `think_timer: Timer`, `strafe_sign: f32`, `strafe_timer: f32`, and `shed_boosters: bool`.
- **Scorers**: Pure, decoupled scorer functions returning `Option<(Tactic, f32)>` in `0.0..=1.0` from a read-only `Context` (`Capability`, `Readiness`, `Skill`, `TeamPlan`, `AiConfig`):
  - `Engage { target, range, posture }` (`Posture::SpinalAim | Orbit | Kite | Brawl`)
  - `Recharge { threat, safe_range }` (energy/capacitor/battery recovery)
  - `Retreat { threat }` (critical HP or completely disarmed)
  - `Support { ally, kind }` (`Overdrive` within `booster.radius` | `Screen` near VIP)
  - `Regroup { position }` (isolated & outnumbered)
  - `Investigate { position }` (shared `TeamAlert` or `last_known` contact)
  - `Search { waypoint }` (expanding patrol around spawn when no intel exists)
- Active tactic receives a hysteresis bonus (`ai_cfg.hysteresis`) to prevent oscillation.
- **Active Booster Power-Shedding**: When `skill >= power_management_level`, if `power_supply` and battery ratio drop below `booster_shed_threshold`, AI inserts `Disabled` on its own `Booster` blocks (`shed_boosters = true`) and removes `Disabled` once battery/supply recover above `booster_restore_threshold`.

### 6. Decoupled `Steering` & Turret `FireOrder`
- **Choice**: Replace `ShipBehavior` with `Steering { desired_heading: Option<f32>, thrust_world: Vec2, throttle_cap: f32 }`.
  - In `update_ship_steering` (`SimulationSet::Control`): PD torque controller aligns ship heading to `desired_heading` (or `thrust_world` angle when `None`). Linear force applies `thrust_world` clamped to `<= 1.0`, scaled by `thrust_power * throttle_cap`; below `omnidirectional_thrust_level`, `thrust_world` is projected onto `forward` (`local_y()`) so low-skill AI flies forward-only while mid/high-skill AI uses full player-legal omnidirectional thrust.
  - Supports spinal lead aiming (`desired_ship_angle(lead_dir, mount_angle)`), perpendicular spinal-line dodging (`spinal_dodge_level`), 4-quadrant armor angling (`armor_angling_level`), and drain-beam throttle capping (`power_management_level`).
- **Turret `FireOrder`**: Ship AI writes `FireOrder { primary_target: Option<Entity>, lead_accuracy: f32 }` on the ship entity (player-controlled ships get `lead_accuracy = 1.0, primary_target = None`).
  - `update_turret_target` prefers `primary_target` when in range for `Closest`/`MostHP` turrets (leaving `EnemyBullet` PD autonomous) and multiplies target velocity by `lead_accuracy` in `lead_point`.
  - `turret_tracking` rotates barrels toward `state.aim_point` instead of raw `ship.core_position`, fixing moving-target lag for rotating turrets.

### 7. Module Layout (`AGENTS.md` Compliant)
```
core/src/content/ship/ai/
├── mod.rs          # AiPlugin, chained SimulationSet::Ai systems
├── component.rs    # Doctrine, Skill, Capability, Role, Readiness, Sensor, Brain, Tactic, Steering, FireOrder, TacticalBoard, TeamPlan, TeamIntel
├── config.rs       # AiConfig / AiConfigValue asset loader (core/assets/config/base/ai.ron) + validation
├── capability.rs   # Pure Feature introspection & live readiness helpers
├── sensor.rs       # Local radar scan, DamageEvent alerts, TeamIntel maintenance
├── board.rs        # Per-team TeamPlan (focus_target, vip_ally, rally_point)
├── scorer.rs       # Modular Utility scorers + hysteresis + Booster power-shedding
└── steer.rs        # Tactic -> Steering + FireOrder + shared debug geometry helpers
```

## Risks / Trade-offs

- **Scoring Balance & Oscillation** → Mitigate with configurable hysteresis in `base/ai.ron`, unit tests covering every tactic transition, and exposing `Brain.current`, `Skill`, `Doctrine`, and `Steering` in dev ship debug UI/gizmos.
- **Booster `Disabled` Lifecycle on Debris/Despawn** → `Disabled` on detached debris is already handled by `disable-debris-blocks`; power-shedding queries only blocks owned by the active ship (`BlockOwner.body == ship_entity`) and tracks `Brain.shed_boosters` cleanly across save/load.
- **Breaking Schema Bump (`DESIGN_SCHEMA_VERSION = 3`, `SAVE_SCHEMA_VERSION = 7`)** → Rejects v2 designs and v6 battle saves cleanly with explicit version mismatch errors; all 4 shipped starter `.ron` files are migrated in the same change.
- **Circular Module Dependencies (`turret` reading `FireOrder`)** → Place `FireOrder` in `content/ship/ai/component.rs` (re-exported from `content/ship`) so `content/block/feature/turret` reads `FireOrder` via `BlockOwner.body` without creating a dependency cycle (`turret` already depends on `content::ship`).

## Migration Plan

1. Add `core/assets/config/base/ai.ron` and `AiConfig` loader/validation.
2. Implement `content/ship/ai/` modules (`component`, `config`, `capability`, `sensor`, `board`, `scorer`, `steer`, `mod`).
3. Replace `ShipBehavior` with `Steering` in `content/ship/` and update `update_ship_steering`; integrate `FireOrder` and `state.aim_point` tracking in `turret/system.rs`.
4. Update `GameSetting` (`ai_level`), `MissionDef` / `ShipSpawn` (`ai_level`), Settings UI stepper, Editor AI `Doctrine` buttons, and dev ship debug UI/gizmos.
5. Bump `DESIGN_SCHEMA_VERSION = 3` and `SAVE_SCHEMA_VERSION = 7`, update all 4 starter `.ron` designs, and update save/load round-trip tests.
6. Update `/docs/{ai.md, config.md, save.md, ui.md}` and run targeted unit tests + `cargo dev` verification.
