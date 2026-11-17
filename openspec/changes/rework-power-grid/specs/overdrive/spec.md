## ADDED Requirements

### Requirement: Booster block
A block with `Booster { radius, boost, rate }` SHALL be a power consumer drawing `rate` and SHALL boost own-team blocks within `radius` (physics shape query) on any ship by `boost` while powered. `boost` SHALL be validated ≥ 1, `radius` and `rate` ≥ 0 at load.

#### Scenario: Powered booster boosts nearby thruster
- **WHEN** a powered booster with boost 1.5 has an own-team thruster within radius
- **THEN** the thruster's `Boost` is 1.5

#### Scenario: Enemy blocks ignored
- **WHEN** an enemy block is within a powered booster's radius
- **THEN** its `Boost` is unaffected by that booster

#### Scenario: Allied ship block
- **WHEN** a powered booster on ship A covers a turret on allied ship B
- **THEN** the turret is boosted and its extra demand is drawn from ship B's grid

### Requirement: Binary booster with priority power
Boosters SHALL be powered before the proportional split, ordered by ascending distance to their ship's core block (ties by entity order). A booster SHALL be powered only if the remaining available power covers its full `rate`; otherwise it SHALL provide no boost and draw nothing that tick.

#### Scenario: Not enough for all boosters
- **WHEN** a ship has 10 available power and two boosters with rate 6, the closer one first
- **THEN** the closer booster is powered and the farther one is not

#### Scenario: Booster starves consumers
- **WHEN** available power equals one booster's rate
- **THEN** the booster is powered and every other consumer has supply 0 unless batteries cover them

### Requirement: Boost stacking and reset
Every boostable block (thruster, turret, generator) SHALL have `Boost` reset to 1 each tick before application. When covered by several powered boosters, its `Boost` SHALL be the maximum `boost` among them.

#### Scenario: Strongest wins
- **WHEN** a block is covered by powered boosters with boost 1.3 and 1.6
- **THEN** its `Boost` is 1.6

#### Scenario: Booster loses power
- **WHEN** the only booster covering a block becomes unpowered
- **THEN** the block's `Boost` is 1 that tick

### Requirement: Overdrive targets
Boost SHALL scale thruster force and torque, turret charge speed and cooldown, beam cooldown, and generator output. Boosted consumers SHALL draw demand multiplied by boost. Boost SHALL NOT affect battery capacity, turret damage or range.

#### Scenario: Turret damage unchanged
- **WHEN** a turret is boosted by 2
- **THEN** its projectile damage and range are unchanged
