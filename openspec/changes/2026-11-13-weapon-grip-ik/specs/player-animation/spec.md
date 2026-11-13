## MODIFIED Requirements

### Requirement: Dual-hand weapon IK
While a weapon is mounted and the player is alive, the presentation layer SHALL resolve a per-hand grip
from the weapon profile and solve both arms onto it: the right chain to the primary grip and the left chain
to the support grip, with optional configurable shoulder settling. Each hand SHALL use its own position,
finger direction, palm direction, elbow pole, wrist trim, and finger curl. IK SHALL clamp unreachable
targets, MUST NOT modify simulation state, and SHALL be skipped when no weapon is mounted or the player is
dead. A weapon whose grip sets `support` to `None` SHALL leave the left arm on its animated pose.

#### Scenario: Both hands stay on the weapon

- **WHEN** the player aims, walks, fires, or reloads with a weapon mounted
- **THEN** the right hand remains on the primary grip and the left hand remains on the support grip within the configured tolerance

#### Scenario: Per-hand grip orientation

- **WHEN** the primary and support grips author different finger, palm, elbow, wrist, or align values
- **THEN** each hand applies its own orientation, letting the right palm face the pistol grip and the left palm face the fore-end

#### Scenario: Authored fields merge over socket defaults

- **WHEN** a grip authors only some fields
- **THEN** the omitted fields come from the socket-derived grip and the authored fields replace them one by one

#### Scenario: One-handed weapon

- **WHEN** a weapon's grip sets `support` to `None`
- **THEN** the left arm keeps its animated pose and only the right arm solves to the primary grip

#### Scenario: High and low aim keep the pose readable

- **WHEN** the aim pose raises the weapon above or below chest height within arm reach
- **THEN** both arms solve without inverting joints and the shoulders settle toward the target

#### Scenario: Unreachable socket extends the arm

- **WHEN** a socket target is farther than the arm length
- **THEN** the arm extends toward the target without non-finite transforms or joint inversion

#### Scenario: Unarmed uses locomotion arms

- **WHEN** no weapon is mounted
- **THEN** IK is skipped and the arms follow the locomotion clips

#### Scenario: Death disables IK in the same frame

- **WHEN** the player dies
- **THEN** weapon IK stops and the death clip controls both arms

#### Scenario: Fingers close around the grip

- **WHEN** a grip authors a finger curl or falls back to the catalog curl
- **THEN** the hand bones curl after the arm solve without changing the solved wrist position

### Requirement: Idle pose relaxation
When the player is idle (horizontal speed below `move_speed_threshold`) and alive, the presentation layer SHALL
procedurally relax the idle pose before IK evaluation: reduce the idle clip's torso X rotation by
approximately 50%. The correction SHALL run after animation evaluation and before `solve_player_arms`, and
SHALL apply to both armed and unarmed players.

#### Scenario: Torso relaxes during idle

- **WHEN** the player is idle and alive
- **THEN** the torso X rotation is reduced by approximately 50% compared to the raw idle clip

#### Scenario: Correction runs before IK

- **WHEN** IK solves an armed idle player
- **THEN** the relaxation has already been applied, so the arm solve targets the relaxed torso instead of being corrected afterwards

#### Scenario: No correction while moving

- **WHEN** horizontal speed is at or above `move_speed_threshold`
- **THEN** no idle relaxation is applied and arms follow locomotion/IK normally

#### Scenario: Dead player is skipped

- **WHEN** the player is dead
- **THEN** no idle relaxation is applied and the death clip controls the pose
