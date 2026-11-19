# Starter Roles

## Purpose

The shipped starter fleet includes a long-range bomber and a fast scout so players can experience distinct strategic roles from built-in designs, expressed entirely through block composition and inline AI profiles.

## Requirements

### Requirement: Starter catalog includes a bomber and a scout

The shipped starter designs SHALL include `B2` and `Blackbird` in addition to `Interceptor` and `Raider`. Each SHALL be a v2 `ShipDesign` document with exactly one 3×3 `core`, orthogonally connected non-overlapping blocks, and power generation that covers active consumer demand. Both SHALL use only block definitions already declared in `block.ron`.

#### Scenario: New starters load and validate

- **WHEN** the `B2` and `Blackbird` starter documents are parsed and their grids validated
- **THEN** each reports exactly one core, connected cells, no overlapping cells, and feasible power

#### Scenario: New starters are available in the editor

- **WHEN** the ship editor builds its ship list after a fresh install
- **THEN** `B2` and `Blackbird` appear alongside `Interceptor` and `Raider`

### Requirement: B2 is a long-range bomber

`B2` SHALL have a broad flying-wing silhouette built from existing hull and triangle blocks, SHALL mount two `colaris` turrets and one `foreshadow` spinal railgun in addition to them, and SHALL contain more hull blocks than thrusters. Its AI SHALL begin with `Attack(target: NearestEnemy)`, SHALL flee at critical health, and SHALL keep its distance with `Kite` when an enemy is in range. Relative to `Blackbird`, `B2` SHALL have greater durability, weapon count and weapon reach but lower thrust and turn rate.

#### Scenario: Bomber silhouette and loadout

- **WHEN** `B2` is spawned
- **THEN** its outline is a broad flying wing of existing hull and triangle pieces, with two `colaris` turrets, one `foreshadow` spinal railgun mounted at `(0, 2)` with `R0`, and fewer thrusters than hull blocks

#### Scenario: Bomber stand-off behavior

- **WHEN** the `B2` AI evaluates its rules
- **THEN** it starts with `Attack(target: NearestEnemy)`, applies `Flee(from: NearestEnemy, distance: 800.0)` with an 8-second cooldown when health is below 0.2, and applies `Kite(range: 4500.0)` when an enemy is within 6000.0

#### Scenario: Bomber trade-off

- **WHEN** `B2` and `Blackbird` stats are compared
- **THEN** `B2` has higher effective hull, more weapons and longer weapon range, and lower thrust power and rotate speed

### Requirement: Blackbird is a fast scout

`Blackbird` SHALL have a long slender dart silhouette built from existing hull and triangle blocks, SHALL mount exactly one `laser_turret`, and SHALL carry multiple `thruster_small` blocks and one `booster` with a lighter power plant than `B2`. Its AI SHALL begin with `Chase(target: NearestEnemy)`, SHALL kite at medium range, and SHALL flee when damaged. Relative to `B2`, `Blackbird` SHALL have higher speed and agility but lower durability, firepower and endurance.

#### Scenario: Scout silhouette and loadout

- **WHEN** `Blackbird` is spawned
- **THEN** its outline is a long slender dart of existing hull and triangle pieces, with exactly one `laser_turret`, multiple `thruster_small` blocks, and one `booster`

#### Scenario: Scout skirmish behavior

- **WHEN** the `Blackbird` AI evaluates its rules
- **THEN** it starts with `Chase(target: NearestEnemy)`, applies `Flee(from: NearestEnemy, distance: 1200.0)` with a 5-second cooldown when health is below 0.4, and applies `Kite(range: 2500.0)` when an enemy is within 4000.0

#### Scenario: Scout trade-off

- **WHEN** `Blackbird` and `B2` stats are compared
- **THEN** `Blackbird` has higher thrust power and rotate speed, and lower effective hull, weapon count and stored power capacity

### Requirement: Quick Battle showcases both new roles

The Quick Battle mission SHALL define a spawn anchor and an enemy list only; it SHALL NOT declare a player ship. The player fleet SHALL be the config-driven default fleet (`Interceptor` and `B2` on a fresh install), spawned at the mission anchor. The enemy roster SHALL be resolved from the mission's `enemies` list.

#### Scenario: Quick Battle roster
- **WHEN** Quick Battle is loaded with a fresh default fleet
- **THEN** the player ships resolve to `Interceptor` and `B2` and the enemy ships resolve to the mission's configured designs

#### Scenario: Custom fleet replaces defaults
- **WHEN** Quick Battle is loaded with a persisted custom fleet
- **THEN** the player ships are the custom fleet designs and no default designs are added
