# weapon-roster Specification

## Purpose

Weapon roster: available weapons, their simulation specs and presentation profiles, model/socket
contract, and switching between slots.

## Requirements

### Requirement: Weapon roster defines multiple playable weapons
The simulation SHALL define at least four weapons in `WeaponsConfig`: `assault_rifle`, `shotgun`, `sniper`,
and `rpg`, each with distinct `WeaponSpecData` stats (fire rate, pellets, damage, spread, projectile
speed/radius/lifetime, knockback, magazine size, reload time).

#### Scenario: Roster contains four weapons

- **WHEN** the combat weapons config is loaded
- **THEN** `weapons` contains `assault_rifle`, `shotgun`, `sniper`, and `rpg`

#### Scenario: Stats differ by class

- **WHEN** comparing `sniper` and `rpg` to `assault_rifle`
- **THEN** sniper has higher damage and lower fire rate; rpg has larger projectile radius and lower
  projectile speed

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

### Requirement: RPG uses existing projectile system
The RPG SHALL be implemented as a `WeaponSpecData` with a single slow, large, high-damage projectile. No
new projectile or damage type is required.

#### Scenario: RPG fires one projectile

- **WHEN** the player fires the RPG
- **THEN** exactly one projectile spawns with the configured speed, radius, and damage