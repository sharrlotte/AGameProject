## ADDED Requirements

### Requirement: Each ship is one power grid
Every block owned by a ship SHALL belong to that ship's single power grid. Power SHALL NOT flow between ships.

#### Scenario: Two ships do not share power
- **WHEN** ship A has surplus generation and ship B has a deficit
- **THEN** ship B's supply is computed only from ship B's generators and batteries

### Requirement: Generators produce constant power
A block with `Generator { output }` SHALL add `output × boost` power per second to its ship's generation, where `boost` is the block's `Boost` value from the previous tick.

#### Scenario: Unboosted generator
- **WHEN** a ship has one generator with output 25 and boost 1
- **THEN** the ship's `power_production` is 25

#### Scenario: Boosted generator lags one tick
- **WHEN** a generator becomes boosted to 1.5 during tick N
- **THEN** its contribution is 25 in tick N and 37.5 from tick N+1

### Requirement: Proportional supply
After booster priority draw, every active non-booster consumer on a ship SHALL receive the same supply ratio `clamp(available / demand, 0, 1)`, where `available` is the remaining generation plus stored energy and `demand` is the sum of their `Demand`. When demand is 0 the supply SHALL be 1.

#### Scenario: Surplus supplies everyone fully
- **WHEN** generation is 20 and demand is 9 with an empty battery bank
- **THEN** every consumer has supply 1

#### Scenario: Deficit is split proportionally without waste
- **WHEN** generation is 9, there are no batteries, and consumers demand 1 and 10
- **THEN** both consumers have supply 9/11 and total delivered power is 9

#### Scenario: No demand
- **WHEN** a ship has no active consumers
- **THEN** supply is 1 and `power_demand` is 0

### Requirement: Batteries store surplus and cover deficit
A block with `Battery { capacity }` SHALL add `capacity` to its ship's storage capacity. Surplus power SHALL charge storage up to capacity; deficit SHALL discharge storage before supply drops below 1. Charge and discharge SHALL have no rate limit.

#### Scenario: Surplus charges battery
- **WHEN** generation exceeds demand by 5 per second for 1 second and storage is not full
- **THEN** stored energy increases by 5, clamped to capacity

#### Scenario: Battery covers deficit
- **WHEN** demand exceeds generation by 4 per second and stored energy is at least 4 × dt
- **THEN** every consumer has supply 1 and stored energy decreases by 4 × dt

#### Scenario: Battery runs dry
- **WHEN** stored energy reaches 0 while demand exceeds generation
- **THEN** supply drops to generation / demand

### Requirement: Battery lifecycle
Battery blocks SHALL spawn fully charged. When a battery block is destroyed, its share of the stored energy SHALL be lost and ship capacity SHALL shrink by its capacity.

#### Scenario: Spawn full
- **WHEN** a ship with two batteries of capacity 50 spawns
- **THEN** `power_stored` is 100 and `power_capacity` is 100

#### Scenario: Destroyed battery loses charge
- **WHEN** one of two equal batteries is destroyed while the ship stores 80
- **THEN** `power_capacity` drops by that battery's capacity and `power_stored` drops by 40

### Requirement: Ship power stats
`ShipStats` SHALL expose `power_production`, `power_demand`, `power_supply`, `power_stored` and `power_capacity`, updated every fixed tick. `power_used` and `efficiency` SHALL NOT exist.

#### Scenario: Stats reflect the tick
- **WHEN** the solver runs for a ship with generation 10, demand 20, no batteries
- **THEN** `power_production` = 10, `power_demand` = 20, `power_supply` = 0.5

### Requirement: Fixed tick order
Power SHALL be solved in `FixedFirst` within `SimulationSet::Stats` in this chained order: generation, booster priority and boost application, consumer demand, proportional split with battery update. Thruster stats SHALL read the result in `FixedPreUpdate`.

#### Scenario: Demand sees current boost
- **WHEN** a booster becomes powered in tick N
- **THEN** consumers in its radius report boosted demand in tick N

### Requirement: Starvation threshold
A supply at or below 0.01 SHALL disable the affected output (thrust, beam). The threshold SHALL be a single named constant in the power module.

#### Scenario: Starved thruster
- **WHEN** a thruster's supply is 0.005
- **THEN** its thrust and torque contribution are 0
