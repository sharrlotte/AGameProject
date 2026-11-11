# player-rig Specification

## Purpose

The armed low-poly player character asset: retained skeleton, tactical gear, weapon socket, rifle,
rig conventions, and the export contract shared by Blender and the game.

## Requirements

### Requirement: Armed rig retains the existing skeleton

The upgraded player asset SHALL keep the existing 31-bone skeleton and its bone names so existing
locomotion and death clips remain usable. The rig MUST NOT require retargeting of the current clips.

#### Scenario: Existing clips bind after the upgrade

- **WHEN** the upgraded `player.glb` is loaded with the existing idle, walk, run, and death clips
- **THEN** every clip binds to its animation targets and plays without missing-bone warnings

#### Scenario: Bone count stays stable

- **WHEN** the asset validation test inspects the upgraded glTF
- **THEN** the skeleton contains the original 31 deform bones plus the added weapon socket bone

### Requirement: Weapon socket bone

The rig SHALL contain a `WeaponSocket` bone parented to the right hand. The socket SHALL define the
rifle's grip position and orientation relative to the hand.

#### Scenario: Rifle attaches to the hand

- **WHEN** the rifle is parented to `WeaponSocket`
- **THEN** the rifle follows the right hand through aim, reload, and locomotion animation

#### Scenario: Socket is addressable by name

- **WHEN** the presentation layer resolves `WeaponSocket`
- **THEN** exactly one bone with that name exists in the skeleton

### Requirement: Low-poly rifle

The project SHALL include a low-poly rifle with a distinct grip, foregrip, muzzle, and magazine. The
rifle SHOULD read clearly at the game's top-down camera distance and MUST match the stylized low-poly
character.

#### Scenario: Rifle provides an IK target

- **WHEN** the support-hand IK system looks up the `Foregrip` transform
- **THEN** exactly one `Foregrip` child exists on the rifle and it is positioned on the handguard

#### Scenario: Rifle provides a muzzle transform

- **WHEN** presentation spawns a muzzle flash
- **THEN** the `MuzzleSocket` transform of the rifle is used as the visual origin

### Requirement: Rig conventions

The exported rig SHALL use a consistent orientation and scale: the character faces `-Z` at rest, feet
rest at `Y = 0`, the origin lies between the feet, and the export scale is `1.0`.

#### Scenario: No runtime transform hacks are required

- **WHEN** the corrected model is loaded with safety `scale = 1.0`, `y_offset = 0.0`, and `yaw_offset_deg = 0.0`
- **THEN** the character stands on the arena floor and faces the aim direction without an extra yaw offset

#### Scenario: Facing math matches the asset

- **WHEN** the visual is rotated with `Quat::from_rotation_arc(Vec3::NEG_Z, direction)`
- **THEN** the character's chest and rifle point along `direction`

### Requirement: Tactical survivor silhouette

The character SHALL read as a low-poly tactical survivor with a vest, cap or helmet, and a small
backpack, in addition to the rifle. Gear MUST be merged or parented so it deforms with the correct
bones.

#### Scenario: Gear follows animation

- **WHEN** the character walks, aims, and reloads
- **THEN** vest and backpack follow the torso bones and the weapon follows the hand socket without visible detachment

### Requirement: Triangle budget

The upgraded character, including gear but excluding the rifle, SHALL remain under 4,000 triangles.
The rifle SHALL remain under 1,000 triangles.

#### Scenario: Asset validation reports the budget

- **WHEN** the asset validation test runs on `player.glb`
- **THEN** it reports triangle counts for the body and the rifle and fails if either exceeds its configured budget

### Requirement: Animation clip contract

The player glTF SHALL contain clips for idle, forward walk, strafe left, strafe right, aim, reload,
and death, using names referenced by the animation catalog.

#### Scenario: Catalog clip names resolve

- **WHEN** the animation catalog is validated against the player glTF
- **THEN** every required clip role resolves to exactly one named animation

#### Scenario: Clips are in place

- **WHEN** the milestone clips are exported
- **THEN** idle, forward walk, strafe left, strafe right, aim, reload, and death contain no root translation that moves the character away from the origin

### Requirement: Export contract

The asset SHALL be exported as a single glTF binary (`player.glb`) with the skinned mesh, armature,
clips, and rifle in one file, using meters and a `1.0` unit scale.

#### Scenario: Single-file load

- **WHEN** the game loads `models/player.glb`
- **THEN** the scene, skeleton, clips, and rifle are all present without external texture or mesh dependencies

#### Scenario: In-place locomotion

- **WHEN** the walk and strafe clips are played
- **THEN** the character's root stays at the origin and movement is driven only by the simulation

### Requirement: Asset work is previewed before export

Changes that alter the character mesh, gear, rifle, skeleton, or clip content SHALL be previewed with
rendered images for user approval before the final `player.glb` is replaced.

#### Scenario: User approves a rig change

- **WHEN** the rig, gear, rifle, or clip edit is ready
- **THEN** a preview render is produced and the export is written only after approval