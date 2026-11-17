## Requirements

### Requirement: Consumers draw only while active
A block with `Consumer { rate }` SHALL report a `Demand` greater than 0 only while it is active. An inactive consumer SHALL report `Demand` 0 and SHALL NOT reduce other consumers' supply.

#### Scenario: Idle thrusters draw nothing
- **WHEN** a ship's throttle is 0 on both axes
- **THEN** every thruster consumer reports `Demand` 0

### Requirement: Thruster demand follows throttle and boost
A thruster with a `Consumer` SHALL report `Demand = rate × max(throttle.linear, throttle.angular) × boost`.

#### Scenario: Full forward throttle
- **WHEN** throttle is linear 1, angular 0, boost 1, rate 3
- **THEN** demand is 3

#### Scenario: Rotation only
- **WHEN** throttle is linear 0, angular 0.5, boost 1, rate 3
- **THEN** demand is 1.5

#### Scenario: Boosted thruster draws more
- **WHEN** throttle is linear 1, boost 1.5, rate 3
- **THEN** demand is 4.5

### Requirement: Thruster output scales with supply and boost
A thruster's computed thrust and rotate speed SHALL equal `base × supply × boost`. A thruster without a `Consumer` SHALL have supply 1.

#### Scenario: Half supplied thruster
- **WHEN** a thruster with base force F has supply 0.5 and boost 1
- **THEN** its thrust contribution is 0.5 × F

#### Scenario: Unpowered core thruster
- **WHEN** the core thruster has no `Consumer` and the ship has a power deficit
- **THEN** its thrust contribution is its full base value times its boost

### Requirement: Consumer feature in config
`block.ron` SHALL declare consumers as `Consumer((rate: <f32>))`; `rate` SHALL be validated as ≥ 0 at load.

#### Scenario: Negative rate rejected
- **WHEN** a block declares `Consumer((rate: -1.0))`
- **THEN** config load logs a warning and the rate is clamped to 0
