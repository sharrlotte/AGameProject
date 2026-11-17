## ADDED Requirements

### Requirement: Turret power is optional and independent of ammo
A turret config SHALL have an optional `power` field: `Charge((capacity, charge_rate))` for `Single` and `Burst` patterns, `Drain((rate))` for `LaserBeam` and `PullingBeam` patterns, or `None`. Ammo SHALL remain independently optional.

#### Scenario: Unpowered turret
- **WHEN** a turret has `power: None`
- **THEN** it fires according to its pattern and ammo only and reports no demand

#### Scenario: Turret with both ammo and power
- **WHEN** a turret has `ammo: Some(..)` and `power: Some(Charge(..))`
- **THEN** it needs both available ammo and a full charge to fire

#### Scenario: Mismatched power kind
- **WHEN** a `Single` pattern turret declares `power: Some(Drain(..))`
- **THEN** config load logs a warning and the turret spawns unpowered

### Requirement: Shot turret internal battery
A `Charge` powered turret SHALL hold an internal charge that spawns full. While charge is below `capacity` it SHALL report `Demand = charge_rate × boost` and gain `charge_rate × supply × boost × dt` per tick, clamped to `capacity`. While full it SHALL report `Demand` 0. The internal charge SHALL NOT be part of grid storage.

#### Scenario: Full battery draws nothing
- **WHEN** a turret's charge equals capacity
- **THEN** its demand is 0

#### Scenario: Starved turret charges slower
- **WHEN** a turret with charge_rate 10, capacity 20 has supply 0.5 and boost 1 from empty
- **THEN** it reaches full charge after 4 seconds

### Requirement: Shot turret firing rule
A `Charge` powered turret SHALL fire only when its charge equals `capacity` AND `fire_rate / boost` seconds have elapsed since its last shot. Firing SHALL reset charge to 0. For `Burst` patterns one full charge SHALL pay for the whole burst.

#### Scenario: Cooldown is the bottleneck
- **WHEN** charge is full but fewer than `fire_rate / boost` seconds have passed
- **THEN** the turret does not fire

#### Scenario: Charge is the bottleneck
- **WHEN** the cooldown elapsed but charge is below capacity
- **THEN** the turret does not fire

#### Scenario: Burst consumes one charge
- **WHEN** a burst turret with full charge starts a burst of 6
- **THEN** all 6 shots fire and charge is 0 after the first shot

### Requirement: Beam turret continuous drain
A `Drain` powered beam turret SHALL report `Demand = rate` while its beam is active and 0 otherwise. Beam damage and pull strength SHALL be multiplied by supply. The beam SHALL stop when supply is at or below the starvation threshold. The beam cooldown SHALL be divided by boost.

#### Scenario: Half supplied beam
- **WHEN** a laser beam with damage D fires at supply 0.5
- **THEN** it deals 0.5 × D per hit tick

#### Scenario: Starved beam stops
- **WHEN** supply drops to 0.005 while the beam is active
- **THEN** the beam stops

#### Scenario: Boosted beam cooldown
- **WHEN** a beam with cooldown 10 has boost 2
- **THEN** its next beam can start 5 seconds after the previous one ended

### Requirement: Legacy power gate removed
Turret firing SHALL NOT be gated by a full-supply threshold (the former `0.999` check).

#### Scenario: Partially supplied turret still fires
- **WHEN** a `Charge` turret has supply 0.3 and eventually reaches full charge
- **THEN** it fires
