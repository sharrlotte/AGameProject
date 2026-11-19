# Smart Utility AI

## Purpose

Block-introspective live `Capability` analysis, `Skill(1..=10)` difficulty curves in `base/ai.ron`, modular Utility AI (`Brain` + `Scorer`s with hysteresis), power/ammo/support block tactics, and decoupled `Steering`.

## Requirements
### Requirement: Pure feature-based live ship capability introspection
Every ship SHALL maintain a `Capability` component computed strictly from its live `BlockGrid` and `BlockDef.features` (with zero hardcoded block type names), recomputed on spawn and on `ShipChanged`. `Capability` SHALL capture fixed vs. rotating turret inventory (including mounted `Rot90` angles and `aim_tolerance`), weapon archetypes (`Single`, `Burst`, `LaserBeam`, `PullingBeam`, `Missile`, `Bullet`, `Laser`, `EnemyBullet` point defense), DPS-weighted optimal and maximum ranges, energy vs. ammo dependency, `Booster` overdrive support, `Generator`/`Battery` presence, 4-quadrant hull armor HP, mobility (`thrust_to_mass`, `can_move`), live `max_radar_range`, and inferred `Role`.

#### Scenario: Losing a fixed railgun dynamically updates capability and role
- **WHEN** a ship with a fixed `turn_speed == 0.0` long-range weapon and short-range rotating turrets loses its fixed weapon block via `BodyMutation::DestroyBlock`
- **THEN** `Capability` recomputes on `ShipChanged`, removes the fixed mount, lowers `optimal_range` and `max_radar_range`, and transitions the inferred `Role` from `Sniper` to a rotating-turret combat role

#### Scenario: Dedicated support hull is auto-detected
- **WHEN** a ship with `Doctrine::Auto` carries `Booster`, `PullingBeam`, or `EnemyBullet` point-defense blocks with offensive DPS below its support weight threshold
- **THEN** `Capability` infers `Role::Support`

#### Scenario: Armed warship with a self-booster stays in combat role
- **WHEN** a standard combat ship with `Doctrine::Auto` carries offensive turrets alongside a `Booster`
- **THEN** `Capability` infers a combat role (`Brawler`, `Skirmisher`, or `Sniper`) rather than `Role::Support`

### Requirement: Continuous Skill(1..=10) difficulty backed by hot-reloadable ai.ron
Every AI ship SHALL carry a `Skill(u8)` component clamped to `1..=10`. All continuous skill curves (`think_rate_hz`, `lead_accuracy`, `aim_jitter_rad`, `range_precision`, `evasion_strength`, `memory_secs`), feature unlock thresholds, and scoring weights SHALL be loaded and validated from `core/assets/config/base/ai.ron` with hot-reload support.

#### Scenario: Skill resolves in layered precedence
- **WHEN** a ship spawns from a `ShipSpawn`
- **THEN** its `Skill` resolves in order: `ShipSpawn.ai_level` -> (if `team.is_player()` then `ai_cfg.default_ally_level`, else `MissionDef.ai_level` -> `GameSetting.ai_level` -> `ai_cfg.default_enemy_level`), clamped to `1..=10`

#### Scenario: Higher Skill increases think rate and unlocks advanced tactics
- **WHEN** comparing a `Skill(2)` ship against a `Skill(9)` ship under the same `base/ai.ron`
- **THEN** `Skill(9)` evaluates faster, applies higher `lead_accuracy`, and unlocks omnidirectional thrust, power management, focus fire, strafe evasion, spinal dodging, and 4-quadrant armor angling

#### Scenario: Invalid ai.ron configuration fails loudly
- **WHEN** `base/ai.ron` contains an out-of-range skill level (outside `1..=10`), inverted curve range, or non-positive think rate
- **THEN** `AiConfig::validate` returns a descriptive error and logs it

### Requirement: Modular Utility AI with hysteresis
At the ship's `Skill`-scaled think rate in `SimulationSet::Ai`, the AI SHALL evaluate composable `Scorer`s for `Engage`, `Recharge`, `Retreat`, `Support`, `Regroup`, `Investigate`, and `Search`, apply a configurable hysteresis bonus to the active `Tactic`, and store the winning `Tactic` in `Brain`.

#### Scenario: Hysteresis prevents rapid tactic oscillation
- **WHEN** a competing tactic scores marginally higher than the active tactic by less than `ai_cfg.hysteresis`
- **THEN** `Brain` retains the current `Tactic`

#### Scenario: Critically damaged or disarmed ship retreats
- **WHEN** a ship's `hp_ratio` drops below its retreat threshold or all its weapons are out of ammo with no energy weapons remaining
- **THEN** the `Retreat` scorer outscores `Engage` and `Brain` transitions to `Tactic::Retreat`

### Requirement: Skill-gated power management and active Booster power-shedding
When `Skill >= power_management_level`, the AI SHALL conserve thruster throttle while firing drain beams under power deficit, disengage via `Tactic::Recharge` when energy/capacitors are depleted under pressure, and toggle `Disabled` on its own `Booster` blocks during power brownouts, restoring them once power recovers. Loot drops SHALL remain player-only.

#### Scenario: Booster power-shedding prevents weapon and thruster brownout
- **WHEN** a ship with `Skill >= power_management_level` experiences `power_supply` and battery charge below the shed threshold while owning powered `Booster` blocks
- **THEN** the AI inserts `Disabled` on its own `Booster` blocks and removes `Disabled` once battery and supply recover above the restore threshold

#### Scenario: Low-skill ship does not shed boosters or throttle-budget
- **WHEN** a ship has `Skill < power_management_level`
- **THEN** it does not insert `Disabled` on `Booster` blocks and does not apply power throttle capping

### Requirement: Decoupled Steering with Skill-gated omnidirectional thrust
The AI SHALL emit `Steering { desired_heading, thrust_world, throttle_cap }` consumed by `update_ship_steering` in `SimulationSet::Control`. Ships with `Skill >= omnidirectional_thrust_level` SHALL apply omnidirectional linear thrust independently of heading (matching player physics); ships below that threshold SHALL project thrust onto their forward vector. High-skill ships SHALL support spinal-line dodging (`spinal_dodge_level`) and 4-quadrant armor angling (`armor_angling_level`).

#### Scenario: High-skill ship strafe-kites while aiming a fixed weapon
- **WHEN** a ship with `Skill >= omnidirectional_thrust_level` engages with a fixed weapon
- **THEN** `Steering` points `desired_heading` at the target lead angle while applying lateral/reverse `thrust_world` to hold optimal standoff range

#### Scenario: High-skill ship angles healthiest hull quadrant toward threat
- **WHEN** a rotating-turret ship with `Skill >= armor_angling_level` has significantly higher surviving HP on one hull quadrant than its damaged side
- **THEN** `Steering.desired_heading` rotates the healthiest quadrant toward the primary threat while continuing to maneuver
