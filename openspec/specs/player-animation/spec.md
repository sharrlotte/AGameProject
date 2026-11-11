# player-animation Specification

## Purpose

Layered player animation: bone-mask layer resolution, directional locomotion blending, combat stance
and aim overlays, facing rules, runtime support-hand IK, and data-driven clip configuration.

## Requirements

### Requirement: Layered animation graph

The player SHALL be animated by an `AnimationGraph` with four layers: lower-body locomotion, upper-body
action, upper-body additive, and full-body override. Locomotion and action layers SHALL run
simultaneously so an upper-body action can play while locomotion continues.

#### Scenario: Layer graph is built from the catalog

- **WHEN** the player model and animation catalog are loaded
- **THEN** the graph contains a locomotion blend node, an action blend node, an additive blend node, and an override clip node, each configured from the catalog

#### Scenario: Lower body and upper body animate at the same time

- **WHEN** the player is walking and a reload starts
- **THEN** the locomotion layer continues to drive the legs while the action layer drives the upper body

### Requirement: Mask groups are registered from animation targets

The animation system MUST register mask groups using the `AnimationTargetId` components present on the
spawned skeleton, not identifiers reconstructed from bone names. Each mask group SHALL be defined by an
explicit catalog list of bone names. Explicit lists are required because `Hips` is the ancestor of the
spine, so a "root plus descendants" rule would place the whole body in the lower-body group.

#### Scenario: Mask groups match glTF target identifiers

- **WHEN** the player glTF scene has spawned and the graph is built
- **THEN** every mask group target ID matches a live `AnimationTargetId` and no configured bone is silently skipped

#### Scenario: Action layer excludes lower-body bones

- **WHEN** an upper-body action clip has non-zero weight
- **THEN** no bone in the lower-body mask group receives values from the action layer

### Requirement: Directional locomotion blending

The locomotion layer SHALL produce movement in any of eight directions from up to four cardinal clips
(forward, backward, left, right). Directional weights MUST be computed in character-local space by
projecting the normalized movement direction onto each cardinal axis, discarding negative
contributions, and normalizing the remaining weights.

#### Scenario: Diagonal movement blends two cardinal clips

- **WHEN** the movement direction is a 45-degree diagonal and the forward and left clips exist
- **THEN** the forward and left clips receive equal non-zero weights and the other cardinal clips receive zero weight

#### Scenario: Missing cardinal clip falls back safely

- **WHEN** a direction has no authored clip in the milestone set
- **THEN** the blend uses the closest available cardinal clips and never produces a zero total weight while the player is moving

#### Scenario: Idle is selected below the speed threshold

- **WHEN** horizontal speed is below the configured `move_speed_threshold`
- **THEN** the locomotion layer drives the idle clip at full weight

### Requirement: Locomotion playback speed follows movement speed

Walk and strafe playback speed SHALL scale with horizontal speed relative to the catalog reference
speed, clamped to catalog minimum and maximum values.

#### Scenario: Slower movement slows the walk cycle

- **WHEN** horizontal speed is half of the walk reference speed
- **THEN** the locomotion playback speed is reduced proportionally and remains within the configured clamp

### Requirement: Aim stance drives the upper-body action layer

The action layer SHALL play the aim clip while the player is in combat stance, fade it out when the
stance ends, and crossfade between aim and reload without stopping locomotion.

#### Scenario: Aim is raised on stance entry

- **WHEN** `CombatStance.aiming` becomes true and no reload is active
- **THEN** the action layer crossfades to the aim clip over the configured action blend time

#### Scenario: Aim is lowered on stance exit

- **WHEN** `CombatStance.aiming` becomes false
- **THEN** the action layer fades to zero weight and the upper body returns to the locomotion clip's arm motion

### Requirement: Reload overlay plays over locomotion

The reload clip SHALL play on the action layer while preserving the current locomotion state, and its
playback SHALL be driven by the simulation reload timer rather than by clip completion.

#### Scenario: Reload while walking

- **WHEN** a reload starts while the player is walking
- **THEN** the reload clip plays on the upper body, the walk cycle continues on the lower body, and the body faces the aim direction

#### Scenario: Reload continues while sprinting

- **WHEN** the player starts sprinting during an active reload
- **THEN** the reload clip continues without restarting and the locomotion layer switches to sprint

#### Scenario: Reload playback matches the reload duration

- **WHEN** the reload duration is modified by a reload-speed upgrade
- **THEN** the reload clip's normalized playback position matches `1 - reload_timer / reload_total` for the whole reload

#### Scenario: Cancelled reload returns to aim

- **WHEN** the simulation reports a cancelled reload
- **THEN** the action layer crossfades from the reload clip to the aim clip or to zero weight according to stance

### Requirement: One-shot action clips restart on repeated triggers

A repeated action trigger MUST restart its clip from the beginning instead of resuming the previous
playback position.

#### Scenario: Second reload starts from frame zero

- **WHEN** a new reload starts after a previous reload finished or was cancelled
- **THEN** the reload clip seeks to its start before the crossfade begins

### Requirement: Facing rules

The player visual SHALL face the aim direction by default and SHALL face horizontal velocity while
sprinting above the movement threshold. Sprint facing SHALL win even when aim is held.

#### Scenario: Aim faces the cursor while strafing

- **WHEN** the player moves sideways without sprinting
- **THEN** the visual body faces the aim direction, not the movement direction

#### Scenario: Sprint faces movement

- **WHEN** the player sprints above the movement threshold while holding the aim button
- **THEN** the visual body faces the horizontal velocity direction

#### Scenario: No valid aim keeps previous facing

- **WHEN** the aim direction is zero or unavailable
- **THEN** the previous visual facing is retained instead of snapping to a default axis

#### Scenario: Facing blends back after sprint

- **WHEN** sprinting ends and the player remains in combat stance
- **THEN** facing returns to the aim direction over the configured blend duration

### Requirement: Runtime support-hand IK

The presentation layer SHALL provide a two-bone analytic IK solver that places the left (support) hand
on the rifle foregrip while the player is in combat stance. IK SHALL be disabled when the stance ends
or the player is dead, and SHALL NOT modify simulation state.

#### Scenario: Support hand follows the foregrip while aiming

- **WHEN** the player is in combat stance and the rifle foregrip is within arm reach
- **THEN** the support hand's world position is within the configured tolerance of the foregrip target

#### Scenario: IK is off outside combat stance

- **WHEN** the player is walking without aiming, firing, or reloading
- **THEN** the IK solver is skipped and the arms follow the locomotion clip

#### Scenario: Unreachable target does not break the pose

- **WHEN** the foregrip target is farther than the arm length
- **THEN** the arm extends toward the target without inverting joints or producing non-finite transforms

#### Scenario: Facing changes do not desynchronize the grip

- **WHEN** the player turns while aiming
- **THEN** the IK solution is recomputed in the same frame and the hand remains on the foregrip

### Requirement: Death overrides all layers

A dead player SHALL play the death clip on all bones, suppress locomotion and action weights, and
disable hand IK.

#### Scenario: Death cancels reload and locomotion

- **WHEN** the player dies during a reload
- **THEN** the death clip reaches full weight, reload and locomotion reach zero weight, and IK is disabled

### Requirement: Data-driven animation catalog with hot reload

Clip roles, layer membership, masks, blend times, speed clamps, IK chain names, and socket names SHALL
be loaded from a `.ron` animation catalog. Editing the catalog while the game runs SHALL rebuild the
animation graph and reset playback state without restarting the application.

#### Scenario: Catalog hot reload rebuilds the graph

- **WHEN** the animation catalog asset is modified during gameplay
- **THEN** the graph is rebuilt from the new catalog, masks are re-registered, and playback restarts from a consistent state

#### Scenario: Missing clip is reported

- **WHEN** a clip name in the catalog does not exist in the player glTF
- **THEN** the system logs a warning naming the clip and the available clips, and continues with the remaining graph

#### Scenario: Parse failure is caught by tests

- **WHEN** the workspace test suite runs
- **THEN** the animation catalog file is deserialized and its required clip roles and mask bones are asserted to be present

### Requirement: Animation systems run only when needed

Animation, IK, and facing systems SHALL run only in `Playing` and `GameOver` states and SHALL NOT run
during loading or menus.

#### Scenario: Menus do not animate the player

- **WHEN** the game is in a menu state
- **THEN** no player animation director, IK, or facing system executes