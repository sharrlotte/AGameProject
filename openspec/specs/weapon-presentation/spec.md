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

### Requirement: Data-driven weapon model space and grip
The unified `WeaponDefinition`'s presentation section SHALL declare the model space each weapon was authored
in, and SHALL map the model into weapon space with `model_space`, optional `model_rotation_deg`, and
`model_scale`. The presentation section MAY override socket transforms in model space and MAY declare a grip
with per-hand position, finger direction, palm direction, elbow hint, finger curl, wrist trim, and
weapon-space alignment. Omitted grip fields SHALL be derived from the weapon's sockets, so a new weapon needs
only catalog data and no engine change.

#### Scenario: Converted model maps into weapon space

- **WHEN** a weapon declares `model_space: ForwardNegYUpZ`
- **THEN** the model is rotated so `-Y` model forward becomes `-Z` weapon forward and `+Z` model up becomes `+Y` weapon up

#### Scenario: Socket override replaces the baked socket

- **WHEN** a profile lists a socket override
- **THEN** the resolved socket node uses the authored model-space transform instead of the one baked into the GLB

#### Scenario: New weapon works without a grip entry

- **WHEN** a weapon omits `grip`
- **THEN** the resolver derives a primary grip from the `Grip` socket and a support grip from `Foregrip`, or no support grip when the foregrip is missing or too close

#### Scenario: Support grip can be disabled

- **WHEN** a profile sets `support: None`
- **THEN** the weapon is treated as one-handed and the left arm is not solved to it

#### Scenario: Per-hand grip is independent

- **WHEN** a profile authors different primary and support finger, palm, elbow, wrist, or align values
- **THEN** each hand resolves and solves with its own values

#### Scenario: Invalid grip data is rejected

- **WHEN** a grip contains a non-finite value, a zero-length direction, or a finger curl beyond 120 degrees
- **THEN** catalog validation fails and the runtime keeps the previous valid configuration

### Requirement: Presentation consumes weapon state through events and presentation-scoped state
The presentation layer SHALL drive visual weapon kick, sway, and model mounting from weapon definitions and combat events, while HUD and UI components observe authoritative simulation state (`WeaponInventory`) directly rather than tracking duplicate magazine ammo and independent reload clocks in `PlayerWeaponRuntime`.

#### Scenario: Firing consumes recoil through events
- **WHEN** a `WeaponFiredMessage` is received
- **THEN** the presentation layer applies the recoil kick to `WeaponPoseState` without decrementing a shadow magazine count

#### Scenario: Visual weapon poses read authoritative inventory
- **WHEN** the weapon pose and arm IK are evaluated
- **THEN** active weapon definitions and reload status are referenced from authoritative simulation state (`WeaponInventory`)

#### Scenario: HUD displays real simulation ammo across weapon swaps
- **WHEN** a player swaps weapons after firing several rounds
- **THEN** the HUD displays the actual remaining ammo of the swapped-to slot in `WeaponInventory` without falsely resetting to full magazine