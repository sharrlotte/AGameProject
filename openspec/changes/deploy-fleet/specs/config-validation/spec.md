## MODIFIED Requirements

### Requirement: Missions and starter designs are validated

Mission config SHALL reference existing designs and valid enemy teams; missions SHALL NOT declare a player team or player ship. Starter designs SHALL have exactly one core, connected blocks, and no overlapping cells.

#### Scenario: Missing mission design
- **WHEN** a mission references a missing design
- **THEN** loading reports an error and does not select the first ship

#### Scenario: Mission declares a player ship
- **WHEN** a mission definition contains a `player` field
- **THEN** load fails with an error naming that field because the field is no longer part of the schema

## ADDED Requirements

### Requirement: Fleet configuration is validated

`game.ron` SHALL supply a non-negative `fleet_spacing`. Every `default_fleet` entry SHALL match a design name available in `ShipCatalog`; an unmatched name SHALL be warned and skipped without preventing startup.

#### Scenario: Negative spacing rejected
- **WHEN** `game.ron` sets `fleet_spacing` below zero
- **THEN** loading reports a config error naming `config/game.ron`

#### Scenario: Unknown default design skipped
- **WHEN** `default_fleet` names a design that is not in the catalog
- **THEN** startup continues with the remaining designs and logs a warning naming the missing design
