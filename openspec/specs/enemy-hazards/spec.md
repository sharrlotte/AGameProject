# enemy-hazards Specification

## Purpose

Ground hazard zones: authored zone specs, fixed-cadence damage ticks, capacity limits with oldest-first
retirement, configuration validation, and ground-marker presentation.

## Requirements

### Requirement: Ground hazard zones

The simulation SHALL represent ground hazards as run-scoped zones with an authored radius, damage per
tick, tick interval, lifetime, and damage kind. A zone MUST be spawned from an authored hazard spec,
MUST live at ground height, and MUST despawn when its lifetime expires or the run ends.

#### Scenario: Zone spawns from a spec

- **WHEN** a producer spawns a hazard with a valid spec
- **THEN** one zone entity exists at the requested horizontal position with the authored radius,
  damage, tick interval, and lifetime

#### Scenario: Zone expires

- **WHEN** a zone's lifetime elapses
- **THEN** the zone is despawned and stops dealing damage

#### Scenario: Zones are run-scoped

- **WHEN** the run ends or the arena scene is exited
- **THEN** every zone is despawned with the run

### Requirement: Hazard damage ticks

A zone SHALL deal damage on a fixed cadence rather than every frame: the first tick MUST occur one
interval after spawn, and each subsequent tick MUST occur one interval later until expiry. Each tick
MUST emit a `DamageMessage` with the zone's authored damage and `DamageKind::Acid` to every eligible
target whose horizontal distance to the zone center is within the radius.

#### Scenario: First tick is delayed by one interval

- **WHEN** a zone spawns under a stationary survivor
- **THEN** no damage is applied before the first tick interval elapses

#### Scenario: Recurring ticks apply the authored damage

- **WHEN** the survivor remains inside a zone across several intervals
- **THEN** each tick applies the authored damage amount

#### Scenario: Targets outside the radius are ignored

- **WHEN** the survivor is farther than the zone radius from its center
- **THEN** the zone applies no damage to the survivor

#### Scenario: Pools do not damage enemies

- **WHEN** an enemy stands inside an acid pool
- **THEN** the pool applies no damage to that enemy

### Requirement: Hazard capacity and retirement

HazardConfig SHALL cap the number of simultaneous zones for a run. When a new zone would exceed the
cap, the oldest zone MUST be retired immediately so the new zone can spawn.

#### Scenario: Cap retires the oldest zone

- **WHEN** a zone is spawned while the active count equals the configured maximum
- **THEN** the oldest zone is despawned and the new zone becomes active

#### Scenario: Under the cap nothing is retired

- **WHEN** a zone is spawned while the active count is below the configured maximum
- **THEN** every existing zone remains active

### Requirement: Hazard configuration validation

HazardConfig MUST reject a non-finite or non-positive tick interval and a maximum active count below
one. Per-zone radius, damage, and lifetime MUST be validated by the behavior that authors the zone.
Invalid configuration MUST be rejected at load and hot reload, and the previous configuration MUST
remain active.

#### Scenario: Invalid hazard config is rejected

- **WHEN** `hazard.ron` contains a zero or non-finite tick interval or a maximum active count of zero
- **THEN** validation fails and the previous configuration stays active

#### Scenario: Authored hazard config parses

- **WHEN** `hazard.ron` is parsed
- **THEN** the caps and tick cadence validate successfully

### Requirement: Hazard presentation

Every active zone SHALL have a visible ground marker that follows the zone's lifetime and disappears
when the zone is despawned. The marker MUST NOT outlive its zone and MUST NOT be attached to the entity
that created it.

#### Scenario: Marker exists while the zone is active

- **WHEN** a zone is alive
- **THEN** a ground marker exists at the zone position

#### Scenario: Marker disappears with the zone

- **WHEN** the zone expires or is retired
- **THEN** its marker is gone with it
