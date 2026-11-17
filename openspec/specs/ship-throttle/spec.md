## Requirements

### Requirement: Ship throttle component
Every ship SHALL have a `Throttle { linear, angular }` component with both values clamped to 0..1, representing movement intent for the current tick.

#### Scenario: Spawned ship is idle
- **WHEN** a ship spawns
- **THEN** its throttle is linear 0, angular 0

### Requirement: Player control writes throttle
While the player pilots a ship, the control system SHALL set `linear = 1` if any of forward, backward or strafe input is held and 0 otherwise, and `angular = 1` if any rotation input is held and 0 otherwise. In FreeLook mode the piloted ship's throttle SHALL be 0.

#### Scenario: Forward and rotate
- **WHEN** the player holds W and A
- **THEN** throttle is linear 1, angular 1

#### Scenario: FreeLook
- **WHEN** the player switches to FreeLook
- **THEN** the previously piloted ship's throttle is linear 0, angular 0

### Requirement: AI behaviour writes throttle
AI-controlled ships SHALL set `linear` to the applied forward force multiplier when forward force is applied and 0 otherwise, and `angular` to the applied torque multiplier when torque is non-zero and 0 otherwise.

#### Scenario: AI approaching inside radius
- **WHEN** an Approach behaviour applies forward force with multiplier 0.25
- **THEN** throttle linear is 0.25

#### Scenario: AI stopped
- **WHEN** an AI ship applies no force and no torque
- **THEN** throttle is linear 0, angular 0
