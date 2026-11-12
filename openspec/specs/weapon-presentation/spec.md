# weapon-presentation Specification

## Purpose

Weapon presentation: anchor height, muzzle flash VFX, bullet-muzzle alignment, and model strategy for
the player's held weapon.

## Requirements

### Requirement: Weapon anchor sits at chest height
The weapon anchor offset SHALL place the mounted weapon at chest height during locomotion, not below the
player's center. The catalog `anchor.offset.y` SHALL be raised accordingly.

#### Scenario: Gun is visible at chest height

- **WHEN** the player is standing and a weapon is mounted
- **THEN** the weapon model is visible at chest height, not at the feet or underground

### Requirement: Muzzle flash on fire
When a `WeaponFiredMessage` is received, the presentation layer SHALL spawn a short-lived muzzle flash
VFX at `message.origin` with a lifetime of approximately 0.08 seconds, then despawn it.

#### Scenario: Flash appears on fire

- **WHEN** the player fires a weapon
- **THEN** a muzzle flash appears at the projectile origin and disappears within ~0.08s

#### Scenario: Flash aligns with bullet

- **WHEN** a flash spawns
- **THEN** its position equals `WeaponFiredMessage.origin`

### Requirement: Bullet origin approximates visual muzzle
The sim SHALL compute `muzzle_pos` using `muzzle_offset` and `aim_height` tuned so the projectile
origin is close to the visual muzzle position at the raised anchor height.

#### Scenario: No large gap between bullet and muzzle

- **WHEN** the player fires
- **THEN** the projectile origin is within a small tolerance of the visual muzzle socket position