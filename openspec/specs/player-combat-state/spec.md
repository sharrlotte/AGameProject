# player-combat-state Specification

## Purpose

Aim intent, combat stance, reload lifecycle, and weapon-swap cancellation exposed by the simulation to
presentation and future gameplay systems.

## Requirements

### Requirement: Aim intent input

The local player input system SHALL set `ActionIntent.aim_held` from a held right mouse button and
SHALL clear it when the button is released.

#### Scenario: Right mouse raises aim intent

- **WHEN** the player holds the right mouse button
- **THEN** `ActionIntent.aim_held` is true for the local player

#### Scenario: Releasing right mouse clears aim intent

- **WHEN** the player releases the right mouse button
- **THEN** `ActionIntent.aim_held` is false on the next input gather

#### Scenario: Aim intent is independent of firing

- **WHEN** the player fires without holding the right mouse button
- **THEN** the shot is still processed and `aim_held` remains false

### Requirement: Combat stance is derived in the simulation

The simulation SHALL maintain a `CombatStance` component whose `aiming` flag is true when the player is
holding aim, holding fire, or reloading. The flag SHALL remain true for a configurable hold duration
after the last triggering action ends, so rapid semi-automatic fire does not flicker the stance.

#### Scenario: Aim button enters combat stance

- **WHEN** `aim_held` becomes true
- **THEN** `CombatStance.aiming` becomes true in the same simulation update

#### Scenario: Firing enters combat stance

- **WHEN** the player fires while `aim_held` is false
- **THEN** `CombatStance.aiming` becomes true and remains true for at least the stance hold duration

#### Scenario: Reload enters combat stance

- **WHEN** a reload starts while `aim_held` is false
- **THEN** `CombatStance.aiming` is true for the whole reload and until the hold duration expires after it finishes

#### Scenario: Stance exits after all actions end

- **WHEN** aim is released, firing has stopped, the reload has finished, and the hold duration has elapsed
- **THEN** `CombatStance.aiming` becomes false

#### Scenario: Dead players are not in combat stance

- **WHEN** the player's health reaches zero
- **THEN** `CombatStance.aiming` is false regardless of intent

### Requirement: Reload lifecycle messages

The combat system SHALL emit typed messages for every reload transition: started, finished, and
cancelled. A cancelled message SHALL include a reason of swapped, died, or interrupted.

#### Scenario: Player-initiated reload emits started

- **WHEN** the player presses reload with a partially empty magazine and no active reload
- **THEN** `ReloadStarted` is emitted with the entity and computed duration, and the reload timer is set

#### Scenario: Reload completion emits finished

- **WHEN** the reload timer reaches zero
- **THEN** the active weapon's magazine is refilled and `ReloadFinished` is emitted once

#### Scenario: Reload cannot start without a valid reason

- **WHEN** the magazine is full or a reload is already active
- **THEN** no `ReloadStarted` message is emitted and the active reload is not restarted

#### Scenario: Empty trigger starts an automatic reload

- **WHEN** the player triggers a weapon with an empty magazine
- **THEN** the same reload lifecycle starts and emits `ReloadStarted`, and no projectile is fired

### Requirement: Reload continues while moving and sprinting

An active reload SHALL continue while the player walks, strafes, or sprints. Sprinting MUST NOT cancel
or pause the reload timer.

#### Scenario: Sprint does not interrupt reload

- **WHEN** the player starts sprinting during an active reload
- **THEN** the reload timer continues to decrease and `ReloadCancelled` is not emitted

#### Scenario: Movement does not change reload duration

- **WHEN** the player moves during a reload
- **THEN** the remaining reload time is unaffected by movement speed

### Requirement: Weapon swap cancels reload

Switching to a different weapon slot SHALL cancel an active reload and emit `ReloadCancelled` with the
swapped reason. Switching to the already active slot MUST NOT cancel the reload.

#### Scenario: Swapping mid-reload cancels it

- **WHEN** the player requests a different weapon slot during a reload
- **THEN** the active slot changes, the reload timer is cleared, and `ReloadCancelled { reason: Swapped }` is emitted

#### Scenario: Swapping partially refills nothing

- **WHEN** a reload is cancelled by a weapon swap
- **THEN** the original weapon's ammunition remains unchanged and no partial refill is applied

#### Scenario: Selecting the current slot keeps the reload

- **WHEN** the player requests the slot that is already active during a reload
- **THEN** the reload continues and no cancellation message is emitted

### Requirement: Death cancels reload

Player death SHALL cancel an active reload and emit `ReloadCancelled` with the died reason.

#### Scenario: Dying mid-reload

- **WHEN** the player's health reaches zero during a reload
- **THEN** the reload timer is cleared and `ReloadCancelled { reason: Died }` is emitted once

### Requirement: Reload progress is exposed to presentation

The simulation SHALL expose reload state through `WeaponInventory.reload_timer` and
`WeaponInventory.reload_total` so presentation can compute normalized progress
`1 - reload_timer / reload_total` without duplicating the reload clock.

#### Scenario: Presentation reads exact progress

- **WHEN** presentation computes reload progress for the animation layer
- **THEN** the value matches the simulation timer for the same frame and never exceeds the `0..=1` range

#### Scenario: Reload duration reflects upgrades

- **WHEN** a reload-speed upgrade changes the computed reload duration
- **THEN** `reload_total` reflects the upgraded duration and progress remains normalized

### Requirement: Combat state resets between runs

A new run SHALL start with no aim stance, no active reload, and no carry-over lifecycle messages.

#### Scenario: Deploying a new run

- **WHEN** a new run starts and the player is spawned
- **THEN** `aim_held` is false, `CombatStance.aiming` is false, and `reload_timer` is `None`

#### Scenario: Stale messages do not leak across states

- **WHEN** the game leaves `Playing` for menus or game over
- **THEN** reload lifecycle messages from the previous run are not consumed as if they were new events