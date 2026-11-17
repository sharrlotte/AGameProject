## ADDED Requirements

### Requirement: Producers generate power without fuel

The system SHALL treat `PowerProducer { output }` as a component on core and reactor blocks and SHALL sum outputs into `ShipStats.power_production`; producers SHALL require no fuel or consumable.

#### Scenario: Core plus reactor
- **WHEN** a ship has a core and one reactor
- **THEN** `ShipStats.power_production` is `35.0`

#### Scenario: No producer
- **WHEN** a ship has no producer block
- **THEN** `ShipStats.power_production` is `0.0`

### Requirement: Consumers declare power demand

The system SHALL treat `EnergyConsumer { rate }` as a component on powered blocks and SHALL sum positive rates into `ShipStats.power_demand`. Non-consuming blocks SHALL be unaffected by power efficiency.

#### Scenario: Demand aggregation
- **WHEN** a ship has two turrets and one thruster with rates 5, 5, and 3
- **THEN** `ShipStats.power_demand` is `13.0` and hull blocks are unaffected

### Requirement: Shortage is split equally among consumers

When production is below demand, the system SHALL compute `share = production / consumer_count` and assign each consumer `efficiency = clamp(share / rate, 0.0, 1.0)`. When production is at least demand or there are no consumers, every consumer SHALL have efficiency `1.0`.

#### Scenario: Mixed draw under shortage
- **WHEN** production is 10 and consumers are 5 and 15
- **THEN** efficiencies are `1.0` and `0.333...`, and `power_used` is `10.0`

#### Scenario: No production
- **WHEN** production is 0 and consumers exist
- **THEN** every consumer efficiency is `0.0`

#### Scenario: No consumers
- **WHEN** there are no consumers
- **THEN** ship efficiency is `1.0` and `power_used` is `0.0`

### Requirement: Efficiency scales powered behavior

A consumer with efficiency `e` SHALL apply it to its feature: turret effective fire rate is `base_fire_rate * e`, turret pattern state advances only while active, and thruster force and torque are multiplied by `e`. At `e <= 0.01` the feature SHALL be inactive.

#### Scenario: Half-power turret
- **WHEN** a turret has 50% efficiency
- **THEN** it fires at half its base rate and does not advance its pattern while inactive

#### Scenario: Unpowered thruster
- **WHEN** a thruster has 0% efficiency
- **THEN** it applies no force or torque

### Requirement: Power is visible to the player

The HUD SHALL show the controlled ship's production, demand, and efficiency with a bar; green when production is at least demand, orange for 1–99%, and red for 0% or consumers without a producer. It SHALL update on `ShipChanged` or `PowerChanged`, not every frame. The editor stats panel SHALL show the same values.

#### Scenario: Reactor loss updates the HUD
- **WHEN** a ship loses its reactor
- **THEN** the HUD changes from green to orange or red without a per-frame rebuild

### Requirement: Power validation is explicit

Block config validation SHALL require producer output `>= 0`, consumer rate `> 0`, and the core to be a producer, and SHALL warn when consumers exist without any producer. `PowerConsumer` and `PowerStorage` SHALL not exist.

#### Scenario: Invalid consumer rate
- **WHEN** a config declares a consumer with rate 0
- **THEN** config loading reports a validation error instead of spawning it

### Requirement: Aura efficiency is distinct from power efficiency

The radius buff formerly named `EfficiencyProvider` and `EfficiencyEffect` SHALL be `EfficiencyAura` and `EfficiencyBuff`, and the final feature multiplier SHALL be power efficiency times the aura buff.

#### Scenario: Combined multiplier
- **WHEN** a block is inside an aura and has 50% power efficiency
- **THEN** its final multiplier is `0.5 * aura_multiplier`
