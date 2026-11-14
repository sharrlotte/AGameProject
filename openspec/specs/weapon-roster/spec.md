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
The player SHALL switch weapons by direct slot selection keys (`Digit1` through `Digit5`, corresponding to available roster slots) and by mouse wheel (next/prev slot, wrapping). Switching SHALL emit `WeaponSwappedMessage` and cancel active reloads.

#### Scenario: Direct digit switches slot across full roster
- **WHEN** the player presses a digit key between Digit1 and Digit5
- **THEN** if a weapon exists in that slot index, `active_slot` becomes that index and `WeaponSwappedMessage` is emitted

#### Scenario: Scroll cycles slots

- **WHEN** the player scrolls the mouse wheel
- **THEN** `active_slot` advances (or retreats) by one, wrapping around the slot count

#### Scenario: Swap cancels reload

- **WHEN** a weapon swap occurs during an active reload
- **THEN** the reload timer clears and `ReloadCancelledMessage` is emitted with reason `Swapped`

### Requirement: Piercing profile is authored per weapon class

Every weapon in the roster SHALL author a `pierce` profile. Within the roster, piercing `power` and
`penalty` MUST increase from sidearm through SMG to rifle, and the largest values MUST be reserved for a
future marksman-class weapon. Updating the profile during a run SHALL apply to subsequently fired shots and
MUST NOT change projectiles already in flight.

#### Scenario: Omitted profile keeps legacy behaviour

- **WHEN** an existing weapon entry has no `pierce` block
- **THEN** it loads with zero power and penalty and behaves exactly as before

#### Scenario: Rifle pierces more than a sidearm

- **WHEN** the authored roster is compared across classes
- **THEN** the rifle-class `power` and `penalty` are greater than the SMG values, which are greater than
  the sidearm values

#### Scenario: Marksman headroom is reserved

- **WHEN** the roster is validated
- **THEN** the highest piercing values are allowed only for a marksman-class entry, not for the existing
  sidearm, SMG, or rifle entries

#### Scenario: Hot reload affects new shots only

- **WHEN** a weapon's `pierce` values change in `weapon.ron` during a run
- **THEN** the next spawned projectile uses the new values and projectiles already in flight keep the
  values they were spawned with

