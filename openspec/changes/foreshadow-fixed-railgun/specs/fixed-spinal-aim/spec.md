## ADDED Requirements

### Requirement: Ship-rotation aiming for fixed weapons
AI-controlled ships equipped with a fixed weapon (`turn_speed == 0.0`) SHALL rotate the ship body to align the fixed weapon's mounted facing direction with the target when within engagement range, rather than orbiting broadside (`GoAround`).

#### Scenario: Ship with forward-mounted fixed railgun engages target
- **WHEN** an AI ship with a `turn_speed == 0.0` weapon placed at `Rot90::R0` enters `Attack` or `Chase` within `min_range` of an enemy ship
- **THEN** `apply_ai_behaviors` SHALL issue `ShipBehavior::AimAt` and `update_ship_behavior` SHALL apply torque to point the ship's forward vector toward the target while holding range

#### Scenario: Ship with rotated fixed railgun respects Rot90 orientation
- **WHEN** an AI ship has its fixed weapon placed at `Rot90::R90` (or `R180`, `R270`)
- **THEN** `ShipBehavior::AimAt` SHALL offset the desired ship rotation by the weapon's mounted `Rot90` angle so the weapon's `transform.up()` aligns with the target

### Requirement: Configurable turret firing cone and ship target leading
`TurretComponentConfig` and `Turret` SHALL support an optional `aim_tolerance` (defaulting to `PI / 16.0`), and `update_turret_target` SHALL lead moving `Ship` targets using `lead_point`.

#### Scenario: Custom aim_tolerance gates firing
- **WHEN** a turret specifies `aim_tolerance: Some(0.035)` in `block.ron`
- **THEN** `update_turret_target` SHALL set `state.can_fire = true` only when `angle_error.abs() < 0.035`

#### Scenario: Default aim_tolerance preserves existing behavior
- **WHEN** a turret omits `aim_tolerance` in `block.ron`
- **THEN** `Turret` SHALL default `aim_tolerance` to `PI / 16.0`

#### Scenario: Turret leads moving enemy ship
- **WHEN** `update_turret_target` evaluates a moving enemy ship with non-zero `Velocity`
- **THEN** `candidate.aim` SHALL be computed via `lead_point` using the target ship's core position, linear velocity, and the turret's projectile speed
