# Fixed Spinal Aim

## Requirements

### Requirement: Ship-rotation aiming for fixed weapons
AI-controlled ships equipped with a fixed weapon (`turn_speed == 0.0`) SHALL rotate the ship body via decoupled `Steering { desired_heading, thrust_world, throttle_cap }` to align the fixed weapon's mounted facing direction (`Rot90`) with the target's lead point when engaging, while optionally applying omnidirectional thrust (`Skill >= omnidirectional_thrust_level`) to hold standoff range or strafe.

#### Scenario: Ship with forward-mounted fixed railgun engages target
- **WHEN** an AI ship with a `turn_speed == 0.0` weapon placed at `Rot90::R0` executes `Tactic::Engage` with `Posture::SpinalAim`
- **THEN** `Steering.desired_heading` points the ship's forward vector toward the target lead point and `update_ship_steering` applies PD torque while holding standoff range

#### Scenario: Ship with rotated fixed railgun respects Rot90 orientation
- **WHEN** an AI ship has its fixed weapon placed at `Rot90::R90` (or `R180`, `R270`)
- **THEN** `Steering.desired_heading` offsets the desired ship rotation by the weapon's mounted `Rot90` angle so the weapon's `transform.up()` aligns with the target lead point

### Requirement: Configurable turret firing cone and ship target leading
`TurretComponentConfig` and `Turret` SHALL support an optional `aim_tolerance` (defaulting to `PI / 16.0`). `update_turret_target` SHALL consult the owning ship's `FireOrder { primary_target, lead_accuracy }` for combat turrets (`Closest`, `MostHP`) to prefer `primary_target` when in range and scale target velocity by `lead_accuracy` inside `lead_point` (player-controlled ships default to `lead_accuracy = 1.0`), while `TargetingStrategy::EnemyBullet` point-defense turrets remain autonomous with full lead. `turret_tracking` SHALL rotate barrels toward `state.aim_point` rather than raw `ship.core_position`.

#### Scenario: Custom aim_tolerance gates firing
- **WHEN** a turret specifies `aim_tolerance: Some(0.035)` in `block.ron`
- **THEN** `update_turret_target` SHALL set `state.can_fire = true` only when `angle_error.abs() < 0.035`

#### Scenario: Default aim_tolerance preserves existing behavior
- **WHEN** a turret omits `aim_tolerance` in `block.ron`
- **THEN** `Turret` SHALL default `aim_tolerance` to `PI / 16.0`

#### Scenario: Combat turret follows ship FireOrder and tracks aim_point
- **WHEN** an AI ship sets `FireOrder { primary_target: Some(enemy), lead_accuracy }` and `enemy` is within turret range
- **THEN** `update_turret_target` selects `enemy`, computes `state.aim_point` using `lead_point` with `velocity * lead_accuracy`, and `turret_tracking` rotates the barrel toward `state.aim_point`

#### Scenario: Point-defense turret stays autonomous
- **WHEN** a turret has `TargetingStrategy::EnemyBullet`
- **THEN** it ignores `FireOrder.primary_target` and intercepts enemy projectiles at full lead accuracy
