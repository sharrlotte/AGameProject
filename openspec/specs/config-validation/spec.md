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

Validation SHALL require unique block keys, a runtime feature for every feature config, a core block with group `"core"`, non-empty turret barrels, `range > 0`, `turn_speed >= 0`, `fire_rate > 0`, burst `amount > 0`, producer output `>= 0`, consumer rate `> 0`, storage capacity `> 0`, shape `size > 0`, voxels within size, and no duplicate voxels.

#### Scenario: Empty turret barrels
- **WHEN** a turret declares an empty barrel list
- **THEN** loading reports a validation error naming the block

### Requirement: Missions and starter designs are validated

Mission config SHALL reference existing designs and valid player and enemy teams. Starter designs SHALL have exactly one core, connected blocks, and no overlapping cells.

#### Scenario: Missing mission design
- **WHEN** a mission references a missing design
- **THEN** loading reports an error and does not select the first ship

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
