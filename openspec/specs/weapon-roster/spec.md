# weapon-roster Specification

## Purpose

Weapon roster: available weapons, their simulation specs and presentation profiles, model/socket
contract, and switching between slots.

## Requirements

### Requirement: Weapon roster defines multiple playable weapons
The simulation SHALL define the playable weapon roster through a single unified `WeaponDefinition` owned
by the config foundation, containing at least five weapons — `m16`, `m1911`, `mp5`, `mp7`, and `scar` — each
with distinct stats.

#### Scenario: Roster contains five weapons

- **WHEN** the unified weapon definition is loaded
- **THEN** the definition contains `m16`, `m1911`, `mp5`, `mp7`, and `scar`

#### Scenario: Stats differ by class

- **WHEN** comparing the converted guns
- **THEN** each has distinct fire rate, damage, and magazine size

### Requirement: Unified weapon definition carries presentation with stats
The unified `WeaponDefinition` SHALL provide presentation data — model path, sockets, poses, recoil, reload,
swap, and grip — for each weapon alongside its stats.

#### Scenario: Profiles match roster

- **WHEN** the unified weapon definition is loaded
- **THEN** each weapon has presentation data with its stats

#### Scenario: Missing weapon degrades safely

- **WHEN** a weapon id has no presentation data
- **THEN** the mount system logs a warning and skips mounting without crashing

### Requirement: Weapon switching via slots and scroll
The player SHALL switch weapons by Digit1/Digit2/Digit3 (direct slot) and by mouse wheel (next/prev
slot, wrapping). Switching SHALL emit `WeaponSwappedMessage` and cancel active reloads.

#### Scenario: Digit switches slot

- **WHEN** the player presses Digit2
- **THEN** `active_slot` becomes 1 and `WeaponSwappedMessage` is emitted

#### Scenario: Scroll cycles slots

- **WHEN** the player scrolls the mouse wheel
- **THEN** `active_slot` advances (or retreats) by one, wrapping around the slot count

#### Scenario: Swap cancels reload

- **WHEN** a weapon swap occurs during an active reload
- **THEN** the reload timer clears and `ReloadCancelledMessage` is emitted with reason `Swapped`

