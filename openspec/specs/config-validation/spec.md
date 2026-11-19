# Config Validation

## Purpose

Config and mission loading returns recoverable errors with validation rules and a readable error screen instead of panicking.
## Requirements
### Requirement: Config and mission failures are recoverable

Config and mission loading SHALL return `ConfigState::Error(ConfigError)` with a readable message and error screen instead of panicking. Mission failure SHALL return to Home with a message.

#### Scenario: Malformed block config
- **WHEN** `block.ron` is malformed
- **THEN** the game shows the config error screen and does not panic

### Requirement: Block config is validated

Validation SHALL require unique block keys, a runtime feature for every feature config, a core block with group `"core"`, non-empty turret barrels, `range > 0`, `turn_speed >= 0`, `fire_rate > 0`, burst `amount > 0`, producer output `>= 0`, consumer rate `> 0`, storage capacity `> 0`, and a shape spec that bakes successfully: positive dimensions, rectangular `Rows` using only `#`/`.` with at least one filled cell and no empty border row or column, triangles whose rasterized bounding box equals their declared size, at least one cell, a single Manhattan-1 connected cell set, and a buildable triangle hitbox. Validation errors SHALL name the block.

#### Scenario: Empty turret barrels
- **WHEN** a turret declares an empty barrel list
- **THEN** loading reports a validation error naming the block

#### Scenario: Disconnected shape rejected
- **WHEN** a block's `Rows` describe two cell groups with no shared orthogonal edge
- **THEN** loading reports a validation error naming the block

#### Scenario: Ragged rows rejected
- **WHEN** a `Rows` shape has rows of differing lengths
- **THEN** loading reports a validation error naming the block

#### Scenario: Invalid row character rejected
- **WHEN** a `Rows` cell contains a character other than `#` or `.`
- **THEN** loading reports a validation error naming the block

#### Scenario: Triangle size mismatch rejected
- **WHEN** a triangle's rasterized bounding box is smaller than its declared size
- **THEN** loading reports a validation error naming the block

### Requirement: Missions and starter designs are validated

Mission config SHALL reference existing designs and valid enemy teams; missions SHALL NOT declare a player team or player ship. Starter designs SHALL have exactly one core, connected blocks, and no overlapping cells.

#### Scenario: Missing mission design
- **WHEN** a mission references a missing design
- **THEN** loading reports an error and does not select the first ship

#### Scenario: Mission declares a player ship
- **WHEN** a mission definition contains a `player` field
- **THEN** load fails with an error naming that field because the field is no longer part of the schema

### Requirement: AI profiles are validated

AI validation SHALL reject unknown or invalid conditions and behaviors, invalid priorities, and negative cooldowns.

#### Scenario: Negative cooldown
- **WHEN** a design contains a rule with a negative cooldown
- **THEN** loading rejects the design with a message

### Requirement: Team ids are constrained without panics

Team ids SHALL be validated to be below 16 through a fallible or validated constructor; `assert!` SHALL NOT be used in production team construction.

#### Scenario: Out-of-range team
- **WHEN** a team id of 16 is supplied
- **THEN** the constructor returns an error or config validation rejects it

### Requirement: Error UI is readable

The error screen SHALL name the failing file or config section and the validation message, and SHALL offer a way back to Home.

#### Scenario: Validation failure is explained
- **WHEN** config validation fails
- **THEN** the player sees the file and reason instead of a crash

### Requirement: Game config values are validated

Game config loading SHALL validate numeric ranges that control runtime behaviour. The debris lifetime minimum and maximum SHALL both be positive and the maximum SHALL be greater than or equal to the minimum. A violation SHALL produce the config error screen naming `config/game.ron` and the failed rule instead of loading the world.

#### Scenario: Inverted debris lifetime range
- **WHEN** `game.ron` sets `debris_lifetime_min_seconds` above `debris_lifetime_max_seconds`
- **THEN** loading reports a config error naming `config/game.ron` and the range rule

#### Scenario: Non-positive debris lifetime
- **WHEN** `game.ron` sets a debris lifetime bound of zero or less
- **THEN** loading reports a config error naming `config/game.ron`

#### Scenario: Valid debris range
- **WHEN** `game.ron` sets a positive minimum and a maximum at or above it
- **THEN** the world loads and debris rolls a lifetime inside that range

### Requirement: Fleet configuration is validated

`game.ron` SHALL supply a non-negative `fleet_spacing`. Every `default_fleet` entry SHALL match a design name available in `ShipCatalog`; an unmatched name SHALL be warned and skipped without preventing startup.

#### Scenario: Negative spacing rejected
- **WHEN** `game.ron` sets `fleet_spacing` below zero
- **THEN** loading reports a config error naming `config/game.ron`

#### Scenario: Unknown default design skipped
- **WHEN** `default_fleet` names a design that is not in the catalog
- **THEN** startup continues with the remaining designs and logs a warning naming the missing design

