# Performance

## Purpose

Background batching, damage-text pooling, and cadenced radar and debug updates keep per-frame and ECS costs bounded.

## Requirements
### Requirement: Background rendering is batched

The background SHALL use one procedural shader quad or two to three tiled parallax layers instead of 2,720 individual sprites, and SHALL update only when the camera moves. The entity count SHALL be reduced by at least an order of magnitude.

#### Scenario: Background entity count
- **WHEN** the game scene starts
- **THEN** background root entities are fewer than 100 and the star field still covers the view

### Requirement: Damage text is pooled

Damage numbers SHALL use a pool of reusable `Text2d` entities (target 64) rather than spawning and despawning per hit.

#### Scenario: Bounded damage text
- **WHEN** many hits land
- **THEN** the damage-text entity count stays bounded by the pool

### Requirement: Radar updates on a cadence

Radar SHALL rebuild at most 10 times per second or on `ShipChanged`, not every fixed tick.

#### Scenario: Idle radar
- **WHEN** no ship changes
- **THEN** radar does not rebuild every tick

### Requirement: Debug UI and ammo icons update on change

Debug ship text SHALL be behind a `DebugOverlay` toggle and update on change; ammo warning icons SHALL be children of blocks or repositioned on change, not every tick.

#### Scenario: Debug overlay disabled
- **WHEN** the debug overlay is disabled
- **THEN** no debug text is rebuilt per tick

### Requirement: Registry ordering is cached

`BlockRegistry::iter` SHALL sort once at registration and SHALL NOT sort on every call.

#### Scenario: Stable iteration
- **WHEN** blocks are iterated repeatedly
- **THEN** ordering is stable without repeated sorting

### Requirement: Performance regressions have smoke tests

Tests or debug assertions SHALL cover background entity counts, bounded damage-text pool usage, and absence of per-tick debug and radar rebuilds.

#### Scenario: Performance suite runs
- **WHEN** the performance suite runs
- **THEN** the entity-count, pool-bound, and per-tick rebuild invariants hold
