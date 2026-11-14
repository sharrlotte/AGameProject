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
The simulation and presentation layers SHALL both use `sockets.muzzle` from `WeaponDefinition` as the authoritative 3D origin for projectile spawning and visual muzzle flash VFX, ensuring zero spatial divergence between fired shots and visual effects.

#### Scenario: No large gap between bullet and muzzle
- **WHEN** the player fires
- **THEN** the projectile origin is placed exactly at `sockets.muzzle.offset` transformed into world space

### Requirement: Data-driven weapon model space and grip
The unified `WeaponDefinition`'s presentation section SHALL declare the model space each weapon was authored in, and SHALL map the model into weapon space with `model_space`, optional `model_rotation_deg`, and `model_scale`. Hand attachment positions and orientations SHALL be declared in a unified `hands` structure (primary and optional support), and mechanical reference points (`magazine` and `muzzle`) SHALL be declared in a unified `sockets` structure with `offset` and optional `rotation_deg`. Sockets and hand attachments SHALL be evaluated directly in weapon space without searching or traversing child nodes of the loaded 3D glTF model.

#### Scenario: Converted model maps into weapon space
- **WHEN** a weapon declares `model_space: ForwardNegYUpZ`
- **THEN** the model is rotated so `-Y` model forward becomes `-Z` weapon forward and `+Z` model up becomes `+Y` weapon up

#### Scenario: Sockets resolve from configuration without model nodes
- **WHEN** a weapon is mounted
- **THEN** hand IK targets and weapon sockets resolve directly from `hands` and `sockets` configuration in `weapon.ron` without scanning glTF node names or emitting missing-socket warnings

#### Scenario: One-handed weapons omit support hand
- **WHEN** a weapon profile defines `hands.support: None`
- **THEN** the primary hand solves to `hands.primary.offset` and the left arm does not engage weapon support IK

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