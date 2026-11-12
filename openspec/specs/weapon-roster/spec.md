# weapon-roster Specification

## Purpose

Weapon roster: available weapons, their simulation specs and presentation profiles, model/socket
contract, and switching between slots.

## Requirements

### Requirement: Weapon roster defines multiple playable weapons
The simulation SHALL define at least five weapons in `WeaponsConfig`: `m16`, `m1911`, `mp5`, `mp7`, and
`scar`, each with distinct `WeaponSpecData` stats (fire rate, pellets, damage, spread, projectile
speed/radius/lifetime, knockback, magazine size, reload time).

#### Scenario: Roster contains five weapons

- **WHEN** the combat weapons config is loaded
- **THEN** `weapons` contains `m16`, `m1911`, `mp5`, `mp7`, and `scar`

#### Scenario: Stats differ by class

- **WHEN** comparing the converted guns
- **THEN** each has distinct fire rate, damage, and magazine size

### Requirement: Presentation profile exists for each weapon
For every weapon id in the sim roster, the presentation catalog SHALL contain a weapon profile with the
same id, a model path, socket names, and pose/recoil/reload/swap settings.

#### Scenario: Profiles match roster

- **WHEN** the presentation weapons catalog is loaded
- **THEN** `weapons` contains a profile for each sim weapon id

#### Scenario: Missing profile degrades safely

- **WHEN** a sim weapon id has no presentation profile
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

