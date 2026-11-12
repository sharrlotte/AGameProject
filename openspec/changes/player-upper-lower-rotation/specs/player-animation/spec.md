## ADDED Requirements

### Requirement: Independent upper-body and lower-body rotation
The player presentation layer SHALL track lower-body yaw and upper-body aim yaw independently so the upper body always faces the aim direction while the lower body orients according to movement and human spine twist limits.

#### Scenario: Small aim offset keeps lower body aligned to movement
- **WHEN** the player walks and the angle between movement direction and aim direction is within the configured `max_twist_deg` cone
- **THEN** the lower body faces the movement direction, the locomotion layer plays forward walk, and the spine twists to align the upper body with the aim direction

#### Scenario: Sideways aim offset rotates lower body to strafe
- **WHEN** the player moves at an angle between `max_twist_deg` and `180 - max_twist_deg` relative to the aim direction
- **THEN** the lower body rotates to face the aim direction, spine twist settles toward zero, and the locomotion layer blends sideways strafe clips

#### Scenario: Backward aim offset rotates lower body opposite movement
- **WHEN** the player moves more than `180 - max_twist_deg` away from the aim direction
- **THEN** the lower body faces opposite the movement direction, the locomotion layer plays `walk_back`, and the spine twists by the remaining angle within `±max_twist_deg` to face the aim direction

#### Scenario: Cone boundaries use hysteresis
- **WHEN** the player moves near a cone boundary angle
- **THEN** the active cone does not oscillate until the angle crosses the boundary by at least `hysteresis_deg`

### Requirement: Idle twist threshold and catch-up turn
While the player is stationary below `move_speed_threshold`, the lower body SHALL remain planted until the yaw difference between upper-body aim and lower-body facing exceeds `max_twist_deg`, after which the lower body smoothly rotates toward aim until within `idle_turn_stop_deg`.

#### Scenario: Small idle mouse sweep twists only the upper body
- **WHEN** the player stands idle and aims within `±max_twist_deg` of the current lower-body facing
- **THEN** the lower-body transform stays stationary and only the spine rotates toward the aim direction

#### Scenario: Large idle mouse sweep turns the lower body to catch up
- **WHEN** the player stands idle and aims beyond `max_twist_deg` from the current lower-body facing
- **THEN** the lower body smoothly slerps toward the aim direction until the remaining twist is within `idle_turn_stop_deg`

### Requirement: Procedural spine twist before IK
The presentation layer SHALL apply residual upper-body yaw twist across the configured spine bones in `PostUpdate` after `AnimationSystems` and before `PlayerIkSet`, rotating around the visual root's vertical axis. Spine twist SHALL reset to zero when the player is dead.

#### Scenario: Support-hand IK stays attached during spine twist
- **WHEN** the spine twists toward the cursor while the player is in combat stance
- **THEN** `apply_support_hand_ik` runs after the spine twist and keeps the left hand positioned at `Foregrip`

#### Scenario: Death disables procedural spine twist
- **WHEN** the player dies or the game enters `GameOver`
- **THEN** procedural spine twist is zeroed so the death clip plays without torso distortion

### Requirement: Directional sprint fallback outside forward cone
When sprinting above `move_speed_threshold` in the sideways or backward cone relative to aim, the locomotion layer SHALL play the directional `strafe_l`, `strafe_r`, or `walk_back` blend scaled to sprint playback speed rather than snapping the upper body away from the cursor.

#### Scenario: Sprinting sideways plays fast strafe while upper body aims
- **WHEN** the player sprints perpendicular to the mouse cursor
- **THEN** the upper body continues facing the cursor and the lower body plays the strafe clip at sprint playback speed

#### Scenario: Sprinting backward plays fast backpedal while upper body aims
- **WHEN** the player sprints opposite to the mouse cursor
- **THEN** the upper body continues facing the cursor, the lower body faces aim-side, and `walk_back` plays at sprint playback speed

## MODIFIED Requirements

### Requirement: Facing rules
The player upper body SHALL always face the aim direction when a valid horizontal aim vector is available, including while sprinting, and SHALL retain its previous facing when aim is unavailable. The lower body SHALL orient independently according to the idle twist threshold and moving cone rules, blended over `facing_blend_secs`.

#### Scenario: Upper body faces the cursor while strafing
- **WHEN** the player moves sideways relative to the cursor
- **THEN** the upper body faces the aim direction and the lower body orients for sideways strafing

#### Scenario: Upper body stays on cursor while sprinting
- **WHEN** the player sprints above the movement threshold while aiming in another direction
- **THEN** the upper body remains facing the aim direction while the lower body selects forward sprint or directional sprint fallback by cone

#### Scenario: No valid aim keeps previous facing
- **WHEN** the aim direction is zero or unavailable
- **THEN** both upper and lower visual facings retain their previous orientations instead of snapping to a default axis

#### Scenario: Cone transitions blend smoothly
- **WHEN** the lower-body target switches between forward, sideways, and backward cones
- **THEN** the lower-body rotation slerps smoothly over `facing_blend_secs`

### Requirement: Data-driven animation catalog with hot reload
Clip roles, layer membership, masks, blend times, speed clamps, IK chain names, socket names, and upper-body spine twist parameters (`max_twist_deg`, `hysteresis_deg`, `idle_turn_stop_deg`, `twist_blend_secs`, bone weights) SHALL be loaded from a `.ron` animation catalog. Editing the catalog while the game runs SHALL rebuild the animation graph, refresh spine bone bindings, and reset playback state without restarting the application.

#### Scenario: Catalog hot reload rebuilds the graph
- **WHEN** the animation catalog asset is modified during gameplay
- **THEN** the graph is rebuilt from the new catalog, masks and spine twist bones are re-registered, and playback restarts from a consistent state

#### Scenario: Missing clip is reported
- **WHEN** a clip name in the catalog does not exist in the player glTF
- **THEN** the system logs a warning naming the clip and the available clips, and continues with the remaining graph

#### Scenario: Parse failure is caught by tests
- **WHEN** the workspace test suite runs
- **THEN** the animation catalog file is deserialized and its required clip roles, mask bones, and valid twist configuration are asserted to be present
