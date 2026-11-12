# player-animation Specification

## Purpose
Layered player animation: bone-mask layer resolution, directional locomotion blending, procedural
weapon pose and dual-hand IK, facing rules, and data-driven clip configuration.

## Requirements

### Requirement: Locomotion phase continuity
When the dominant locomotion anchor changes, the incoming clip SHALL seek to the shared normalized locomotion phase before its weight rises, so feet do not pop between anchor clips. The phase SHALL advance from horizontal speed and the dominant clip duration, SHALL NOT advance while idle, and SHALL reset when the graph is rebuilt.

#### Scenario: Diagonal switch preserves foot phase

- **WHEN** the dominant anchor changes from forward walk to forward-right while moving
- **THEN** the incoming clip starts at the same normalized phase as the outgoing clip and the legs do not pop

#### Scenario: Clip durations are normalized

- **WHEN** two anchor clips have different durations
- **THEN** the shared phase maps to each clip's own duration instead of using absolute seconds

#### Scenario: Hot reload resets phase

- **WHEN** the animation catalog is reloaded during gameplay
- **THEN** the locomotion phase restarts from zero together with the rebuilt graph

### Requirement: Camera-relative movement input
Movement input SHALL be interpreted relative to the camera's horizontal forward direction, falling back to world axes when no camera transform is available. Input direction MUST NOT be hardcoded to world X/Z.

#### Scenario: Forward input follows the camera

- **WHEN** the camera yaw changes and the player presses forward
- **THEN** movement follows the camera's horizontal forward direction

#### Scenario: No camera falls back safely

- **WHEN** no camera transform is available
- **THEN** movement uses the world axis fallback and remains finite

### Requirement: Procedural weapon pose
While a weapon is mounted and the player is alive, the presentation layer SHALL pose a runtime weapon anchor parented to the player visual root, using catalog offsets for relaxed and aiming poses plus sway, recoil, reload, and swap offsets. Aim MUST NOT require an aim animation clip, and the weapon MUST NOT be parented to a hand bone.

#### Scenario: Aiming raises the weapon

- **WHEN** the player enters combat stance with a weapon mounted
- **THEN** the weapon anchor blends to the aim pose and the weapon points along the aim direction

#### Scenario: Relaxed pose while not aiming

- **WHEN** the player is not aiming and not reloading
- **THEN** the weapon anchor holds the lowered relaxed pose while locomotion continues

#### Scenario: Missing socket degrades safely

- **WHEN** a weapon asset omits a configured socket
- **THEN** the system logs a warning, falls back to the weapon root for that target, and keeps the pose finite

#### Scenario: Weapon hot reload respawns the mount

- **WHEN** the weapon catalog is modified during gameplay
- **THEN** the mounted weapon visual is respawned, sockets are re-resolved, and the current pose is reapplied

### Requirement: Dual-hand weapon IK
While a weapon is mounted and the player is alive, the presentation layer SHALL solve both arms to the weapon sockets: the right chain to `Grip` and the left chain to `Foregrip`, with optional configurable shoulder settling and per-hand grip orientation offsets. IK SHALL clamp unreachable targets, MUST NOT modify simulation state, and SHALL be skipped when no weapon is mounted or the player is dead.

#### Scenario: Both hands stay on the weapon

- **WHEN** the player aims, walks, fires, or reloads with a weapon mounted
- **THEN** the right hand remains on `Grip` and the left hand remains on `Foregrip` within the configured tolerance

#### Scenario: Per-hand grip orientation

- **WHEN** `right_hand_offset_deg` and `left_hand_offset_deg` are set differently
- **THEN** each hand applies its own rotation offset, allowing right palm to face the pistol grip and left palm to face the fore-end

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

### Requirement: Procedural weapon reactions
Weapon recoil, reload, and swap poses SHALL be derived from simulation messages and reload progress, not from per-weapon upper-body clips. Recoil SHALL decay and clamp; reload SHALL track normalized progress; swap SHALL lower, replace, and raise the weapon over a catalog duration.

#### Scenario: Firing produces a bounded recoil kick

- **WHEN** the player fires and `WeaponFiredMessage` names the player as shooter
- **THEN** the weapon and upper body receive a recoil impulse that decays within the configured recovery time and does not accumulate without bound

#### Scenario: Reload pose tracks simulation progress

- **WHEN** reload progress moves from 0 to 1
- **THEN** the left hand moves to `Magazine`, the magazine drops and returns, and the hand returns to `Foregrip` in step with `reload_progress()` rather than clip completion

#### Scenario: Cancelled reload returns to the grip

- **WHEN** the reload is cancelled
- **THEN** the left hand blends back to `Foregrip` and the weapon returns to its aim or relaxed pose

#### Scenario: Swap lowers, replaces, and raises

- **WHEN** `WeaponSwappedMessage` is received
- **THEN** the old weapon lowers and hides, the new weapon mounts and resolves its sockets, and both hands raise to the current pose


### Requirement: Layered animation graph
The player SHALL be animated by an `AnimationGraph` with a base locomotion layer, an optional upper-body additive action layer, and a full-body override layer. Aim and reload SHALL NOT be animation clip layers; aiming is a procedural weapon pose and arm IK. Locomotion and additive action layers SHALL run simultaneously so an upper-body action can play while locomotion continues.

#### Scenario: Layer graph is built from the catalog

- **WHEN** the player model and animation catalog are loaded
- **THEN** the graph contains a locomotion blend node, an additive action node when action clips exist, and an override clip node, each configured from the catalog

#### Scenario: Lower body and upper body animate at the same time

- **WHEN** the player is walking and an upper-body action is active
- **THEN** the locomotion layer continues to drive the legs while the action layer drives the upper body

#### Scenario: Aim adds no clip layer

- **WHEN** the player enters combat stance
- **THEN** no aim clip node is required and the weapon pose and IK produce the aiming arms

### Requirement: Directional locomotion blending
The locomotion layer SHALL produce movement in any direction from a catalog list of angular anchors `(role, angle_deg)`, measured in character-local space relative to the aim-facing body (`0` forward, `+90` right, `180` backward, `-90` left). The system MUST compute the local movement angle by rotating horizontal velocity into aim space, select the two anchors that bracket that angle, and assign complementary weights that sum to one. At most two anchors SHALL have non-zero weight, and at least four cardinal anchors SHALL ship in the initial catalog. Adding diagonal or finer anchors MUST NOT require Rust changes.

#### Scenario: Diagonal movement blends two cardinal clips

- **WHEN** the catalog has four cardinal anchors and the local movement angle is 45 degrees
- **THEN** the adjacent forward and right anchors receive equal non-zero weights and the other anchors receive zero weight

#### Scenario: Eight-anchor catalog resolves a diagonal exactly

- **WHEN** the catalog contains an anchor at 45 degrees and the local movement angle is 45 degrees
- **THEN** that anchor receives full weight and no other anchor is blended

#### Scenario: Missing anchor clip falls back safely

- **WHEN** an anchor's clip is missing from the player glTF
- **THEN** that anchor is dropped with a warning and the neighboring anchors still produce a valid blend with total weight one while the player is moving

#### Scenario: Single anchor remains valid

- **WHEN** only one anchor resolves at bind time
- **THEN** that anchor receives full weight in every movement direction

#### Scenario: Seam is continuous

- **WHEN** the local movement angle crosses the `+/-180` degree seam
- **THEN** anchor weights change continuously without a pop

#### Scenario: Idle is selected below the speed threshold

- **WHEN** horizontal speed is below the configured `move_speed_threshold`
- **THEN** the locomotion layer drives the idle clip at full weight and does not advance the locomotion phase

### Requirement: Sprint gait

An anchor MAY declare an optional `sprint_role` naming a sprint variant clip. When the simulation reports sprinting and an anchor has a sprint variant, the locomotion layer SHALL crossfade that anchor's walk clip and sprint clip by a smoothed gait blend instead of speeding up the walk clip. Anchors without a sprint variant SHALL fall back to the existing sprint playback multiplier, blended by the same gait factor. The gait blend SHALL be a smoothed 0..1 value tracking `Stamina.is_sprinting` while moving, over `gait_blend_secs`.

#### Scenario: Forward sprint uses the run clip

- **WHEN** the player sprints forward and the forward anchor has `sprint_role: sprint_fwd`
- **THEN** the `sprint_fwd` clip gains weight and the `walk_fwd` clip loses weight over `gait_blend_secs`, and the legs read as a run rather than a fast walk

#### Scenario: Missing sprint variant falls back

- **WHEN** the player sprints with an anchor that has no `sprint_role`
- **THEN** that anchor keeps using its walk clip with the sprint playback multiplier, blended by the gait factor

#### Scenario: Gait blend crossfades smoothly

- **WHEN** sprint is toggled during movement
- **THEN** walk and sprint node weights change continuously over `gait_blend_secs` without a pop

#### Scenario: Phase rate blends during crossfade

- **WHEN** the gait blend is between 0 and 1 and the dominant anchor has a sprint variant
- **THEN** phase advancement uses a duration and reference speed linearly interpolated between the walk and sprint clip values by the gait blend

#### Scenario: Sprint variant seeks to the shared phase

- **WHEN** the dominant anchor changes while sprinting
- **THEN** the incoming sprint clip seeks to `phase * sprint_duration` before its weight rises

#### Scenario: Strict validation of sprint_role

- **WHEN** an anchor sets `sprint_role` to a role that does not exist in the catalog clips
- **THEN** catalog validation fails

#### Scenario: Hot reload resets gait state

- **WHEN** the animation catalog is reloaded during gameplay
- **THEN** the gait blend resets to zero together with the rebuilt graph and phase

### Requirement: Locomotion playback speed follows movement speed
Locomotion playback speed SHALL scale with horizontal speed relative to the catalog reference speed, clamped to catalog minimum and maximum values. An anchor MAY define an additional playback multiplier. While the simulation reports sprinting, anchors without a sprint variant SHALL apply the sprint playback multiplier (blended by the gait factor); anchors with a sprint variant SHALL instead crossfade to their sprint clip per the sprint gait requirement, with phase rate blended between walk and sprint values.

#### Scenario: Slower movement slows the walk cycle

- **WHEN** horizontal speed is half of the walk reference speed
- **THEN** the locomotion playback speed is reduced proportionally and remains within the configured clamp

#### Scenario: Sprinting speeds up anchors without a sprint variant

- **WHEN** the player sprints with a directional anchor that has no `sprint_role`
- **THEN** that anchor plays faster by the sprint multiplier (blended by the gait factor) instead of switching to a movement-facing body

#### Scenario: Sprinting crossfades anchors with a sprint variant

- **WHEN** the player sprints with a directional anchor that has `sprint_role`
- **THEN** that anchor crossfades to its sprint clip per the sprint gait requirement instead of applying the sprint multiplier

### Requirement: Facing rules
The player visual root SHALL face the horizontal aim direction at all times when a valid aim vector exists, including while strafing and sprinting. Movement direction MUST NOT write body yaw. Facing SHALL blend toward the aim direction using shortest-arc smoothing over the configured blend time and SHALL retain its previous yaw when aim is unavailable. Death SHALL stop aim tracking.

#### Scenario: Aim faces the cursor while strafing

- **WHEN** the player moves sideways
- **THEN** the visual body faces the aim direction, not the movement direction

#### Scenario: Sprint keeps the cursor facing

- **WHEN** the player sprints above the movement threshold while holding the aim button
- **THEN** the visual body continues to face the aim direction

#### Scenario: No valid aim keeps previous facing

- **WHEN** the aim direction is zero or unavailable
- **THEN** the previous visual facing is retained instead of snapping to a default axis

#### Scenario: Large flick turns smoothly

- **WHEN** the aim direction changes by 180 degrees in one frame
- **THEN** the visual root rotates over the configured blend time using the shortest arc and does not overshoot

#### Scenario: Death stops aim tracking

- **WHEN** the player dies
- **THEN** the visual root stops tracking aim and the death clip plays without body yaw changes

### Requirement: One-shot action clips restart on repeated triggers
A repeated action clip trigger MUST restart its clip from the beginning instead of resuming the previous playback position.

#### Scenario: Repeated action starts from frame zero

- **WHEN** a new one-shot action clip starts after a previous one finished or was cancelled
- **THEN** the clip seeks to its start before the crossfade begins

### Requirement: Death overrides all layers
A dead player SHALL play the death clip on all bones, suppress locomotion and additive action weights, disable weapon pose and hand IK, and stop aim tracking.

#### Scenario: Death cancels reload and locomotion

- **WHEN** the player dies during a reload
- **THEN** the death clip reaches full weight, locomotion and action weights reach zero, the weapon pose is suppressed, and IK is disabled

### Requirement: Data-driven animation catalog with hot reload
Clip roles, anchor angles, layer membership, masks, blend times, speed clamps, aim pose height, IK chain names, shoulder settle, and socket names SHALL be loaded from hot-reloadable `.ron` catalogs. Anchor angles MUST be unique and within `[-180, 180]` degrees, and at least one anchor MUST exist. Editing the animation or weapon catalog while the game runs SHALL rebuild the affected graph or weapon mount and reset playback state without restarting the application.

#### Scenario: Catalog hot reload rebuilds the graph

- **WHEN** the animation catalog asset is modified during gameplay
- **THEN** the graph is rebuilt from the new catalog, masks and IK chains are re-registered, and playback restarts from a consistent state

#### Scenario: Weapon catalog hot reload respawns the weapon

- **WHEN** the weapon catalog asset is modified during gameplay
- **THEN** the mounted weapon is respawned with the new model and sockets and the current pose is reapplied

#### Scenario: Missing clip is reported

- **WHEN** a clip name in the catalog does not exist in the player glTF
- **THEN** the system logs a warning naming the clip and the available clips and continues with the remaining graph

#### Scenario: Invalid anchors are rejected

- **WHEN** the catalog contains duplicate anchor angles, an angle outside `[-180, 180]`, or no anchors
- **THEN** validation fails the parse test and the runtime keeps the previous valid configuration

#### Scenario: Parse failure is caught by tests

- **WHEN** the workspace test suite runs
- **THEN** the animation and weapon catalogs are deserialized and their required roles, anchor validity, masks, chains, and socket names are asserted to be present

### Requirement: Idle pose relaxation
When the player is idle (horizontal speed below `move_speed_threshold`) and alive, the presentation layer
SHALL procedurally relax the shoulders, arms, and spine after IK evaluation: reduce shoulder abduction
(blend shoulder Z rotation toward zero) and reduce the idle clip's torso X rotation by approximately
50%. The correction SHALL run after `solve_player_arms` and before transform propagation, and SHALL
apply to both armed and unarmed players.

#### Scenario: Shoulders relax during idle

- **WHEN** the player is idle and alive
- **THEN** the shoulder Z rotation is reduced toward zero compared to the raw idle clip

#### Scenario: Correction runs after IK

- **WHEN** IK has written arm rotations for an armed idle player
- **THEN** the relaxation applies on top without breaking grip reach beyond tolerance

#### Scenario: No correction while moving

- **WHEN** horizontal speed is at or above `move_speed_threshold`
- **THEN** no idle relaxation is applied and arms follow locomotion/IK normally

#### Scenario: Dead player is skipped

- **WHEN** the player is dead
- **THEN** no idle relaxation is applied and the death clip controls the pose

### Requirement: Animation systems run only when needed
Animation, weapon pose, IK, and facing systems SHALL run only in `Playing` and `GameOver` states and SHALL NOT run during loading or menus.

#### Scenario: Menus do not animate the player

- **WHEN** the game is in a menu state
- **THEN** no player animation director, weapon pose, IK, or facing system executes
