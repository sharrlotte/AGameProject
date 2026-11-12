## MODIFIED Requirements

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