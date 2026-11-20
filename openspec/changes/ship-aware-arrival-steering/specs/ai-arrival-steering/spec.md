## ADDED Requirements

### Requirement: Ship-aware arrival speed
For `Tactic::Regroup`, `Tactic::Investigate`, `Tactic::Search`, and `Tactic::Support`, the AI SHALL steer toward the destination at a speed derived from that ship's own live acceleration rather than a fixed global ceiling. The deceleration input SHALL be `ShipStats.thrust_power / ShipStats.mass.max(1.0)`, and the desired speed SHALL be the smaller of the braking-limited speed `sqrt(2 × decel × (dist - arrival_radius))` and the ship's full-thrust terminal speed `decel / linear_damping`.

#### Scenario: Fast ship cruises faster than a heavy ship
- **GIVEN** a light 4-thruster ship and a heavy 2-thruster ship in `Tactic::Search`, both farther from their waypoint than the braking distance
- **WHEN** steering is evaluated
- **THEN** each ship's desired speed equals its own `decel / linear_damping`, and the light ship's desired speed is proportionally higher

#### Scenario: Speed ceiling is not mass-independent
- **GIVEN** a ship in `Tactic::Investigate` with a 2400-unit waypoint
- **WHEN** steering is evaluated
- **THEN** the commanded speed is NOT capped by `sqrt(2 × linear_damping × dist)` and the ship is free to accelerate toward its own terminal speed

### Requirement: Active reverse-thrust braking
When the ship's speed toward the waypoint exceeds the braking-limited safe speed, the controller SHALL command negative thrust along the approach direction to brake, scaled by the speed error divided by the ship's own deceleration, clamped to `-1.0..1.0`.

#### Scenario: Overspeed approach brakes
- **GIVEN** a ship inside the braking distance whose current approach speed exceeds `sqrt(2 × decel × (dist - arrival_radius))`
- **WHEN** steering is evaluated
- **THEN** `thrust_world` points opposite the approach direction and `throttle_cap` is `1.0`

#### Scenario: No phase chatter
- **GIVEN** a ship braking toward the arrival boundary
- **WHEN** its speed decreases continuously
- **THEN** the commanded thrust changes continuously without switching between full forward and full reverse in successive ticks

### Requirement: Arrival boundary stops cleanly
When the ship is within `arrival_radius` of the destination, the controller SHALL command zero thrust and `throttle_cap = 0.0`, and SHALL NOT evaluate the square-root braking term.

#### Scenario: Inside arrival radius
- **GIVEN** `dist <= arrival_radius`
- **WHEN** steering is evaluated
- **THEN** `thrust_world` is `Vec2::ZERO`, `throttle_cap` is `0.0`, and no `NaN` is produced

### Requirement: No arrival throttle-cap configuration
The AI arrival controller SHALL NOT read a per-behavior throttle cap, and the `arrive_throttle_cap` and `support_throttle_cap` keys SHALL NOT exist in `ai.ron` or `CombatTuning`.

#### Scenario: Config contains no dead keys
- **GIVEN** the shipped `core/assets/config/base/ai.ron`
- **WHEN** config is deserialized
- **THEN** no `arrive_throttle_cap` or `support_throttle_cap` field is present and deserialization succeeds

## MODIFIED Requirements

### Requirement: Steering pipeline consumes live ship stats
`steer_ships` SHALL read `ShipStats` for every AI ship and pass that ship's `thrust_power` and `mass` into the shared arrival controller; `steer_support` SHALL forward the same deceleration value. The `Steering` output of arrival behaviors SHALL carry `throttle_cap = 1.0` with magnitude encoded in `thrust_world`.

#### Scenario: Support ships use ship-aware arrival
- **GIVEN** an AI ship in `Tactic::Support`
- **WHEN** steering is evaluated
- **THEN** it uses its own deceleration in the shared arrival controller and is not capped by a support-specific throttle value

#### Scenario: Player-controlled ship is unaffected
- **GIVEN** the ship currently controlled by the player
- **WHEN** the AI steering systems run
- **THEN** the ship is skipped as before and no arrival steering is applied
