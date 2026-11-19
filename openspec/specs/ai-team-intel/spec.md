# AI Team Intel

## Purpose

Live `max_radar_range` scanning (`Sensor`), always-on team-wide `TeamIntel` contact/alert sharing with bounded memory, and two-tier `TacticalBoard` coordination.

## Requirements
### Requirement: Live radar scanning bounded by max_radar_range
AI ships SHALL NOT use an omniscient global ship map for targeting. Each ship's `Sensor` SHALL scan enemy ships strictly within its live `Capability.max_radar_range` (computed from surviving blocks and `base_core_radar_range`).

#### Scenario: Enemy outside all teammates' radar ranges is unseen
- **WHEN** an enemy ship is beyond the `max_radar_range` of every ship on team 1 and has not damaged team 1
- **THEN** team 1's `TeamIntel.contacts` does not contain that enemy and team 1 ships run `Search`

#### Scenario: Losing long-range blocks shrinks radar range
- **WHEN** a ship loses its longest-range turret block
- **THEN** its `Sensor` scan range shrinks to its updated `Capability.max_radar_range`

### Requirement: Always-on team-wide intel bus (TeamIntel)
Every team SHALL maintain a shared `TeamIntel` state inside `TacticalBoard`, active at all `Skill` levels (`1..=10`). Whenever any single ship on a team spots an enemy within its `max_radar_range`, that `SharedContact` SHALL immediately be available to all ships on the same team. Whenever any ship on a team takes damage (`DamageEvent`) from an unseen attacker, it SHALL post a shared `TeamAlert` to `TeamIntel`. When a spotted enemy leaves radar or the spotter is destroyed, its last known position and velocity SHALL persist in `TeamIntel.last_known` for `memory_secs`.

#### Scenario: One ship spots the player and the whole enemy team knows
- **WHEN** enemy ship A (team 1) has the player ship within A's `max_radar_range` and enemy ship B (team 1) is outside B's own `max_radar_range`
- **THEN** `TeamIntel` for team 1 contains the player as a live `SharedContact` and enemy ship B immediately reacts to engage

#### Scenario: Spotter destroyed preserves decaying team memory
- **WHEN** enemy ship A spots the player and is subsequently destroyed before ship B gets within radar range
- **THEN** the player remains in team 1's `TeamIntel.last_known` for `memory_secs` and ship B investigates the last known position instead of immediately reverting to idle search

#### Scenario: Unseen long-range hit alerts the team
- **WHEN** the player damages enemy ship A from beyond team 1's radar range
- **THEN** a shared `TeamAlert` is recorded in team 1's `TeamIntel.alerts` so ship A and nearby teammates investigate the attack direction

### Requirement: Two-tier TacticalBoard teamwork
`TacticalBoard` SHALL compute a per-team `TeamPlan` each tick. All `Skill` levels (`1..=10`) SHALL share `TeamIntel`. Ships with `Skill >= focus_fire_level` SHALL prioritize `TeamPlan.focus_target` when scoring and targeting; ships with `Skill >= support_coordination_level` (or explicit `Doctrine::Support`/`Doctrine::Escort`) SHALL pair with `TeamPlan.vip_ally` to maintain `Booster` overdrive aura coverage or screen threats.

#### Scenario: Mid-skill squad focus-fires a shared target
- **WHEN** two team 1 ships with `Skill >= focus_fire_level` have multiple enemies in `TeamIntel.contacts`
- **THEN** `TeamPlan.focus_target` selects one priority enemy and both ships bias `Engage` and `FireOrder.primary_target` toward it

#### Scenario: Support ship holds Booster aura over VIP ally
- **WHEN** a team 1 ship with `Role::Support` carrying a `Booster` has an active allied combat ship (`TeamPlan.vip_ally`)
- **THEN** it executes `Tactic::Support` to stay within `booster.radius` of `vip_ally`
