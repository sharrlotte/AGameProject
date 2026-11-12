## ADDED Requirements

### Requirement: Separate weapon assets
Each weapon SHALL be exported as its own glTF binary under `game/assets/models/weapons/`, containing mesh, materials, and socket nodes but no character armature and no animation clips. Weapon forward SHALL be `-Z`, up `+Y`, units meters, and the model origin SHALL sit at the grip. Each weapon SHALL remain under 1,000 triangles and read clearly at the top-down camera distance.

#### Scenario: Weapon loads without the player

- **WHEN** the presentation layer mounts a weapon asset
- **THEN** the scene contains only mesh, materials, and sockets and loads without requiring the player glTF

#### Scenario: Weapon budget is validated

- **WHEN** the asset validation test inspects a weapon glTF
- **THEN** it reports the triangle count and fails if the weapon exceeds 1,000 triangles

#### Scenario: Player asset excludes the weapon

- **WHEN** `player.glb` is inspected
- **THEN** it contains no rifle mesh and no rifle materials

### Requirement: Weapon socket contract
Every weapon glTF SHALL contain exactly one node named `Grip`, `Foregrip`, `Magazine`, and `Muzzle`. Sockets SHALL be authored in weapon space, unique by name, and positioned on the corresponding physical feature. `Grip` SHALL define the firing hand, `Foregrip` the support hand, `Magazine` the reload hand target and detachable magazine, and `Muzzle` the shot origin.

#### Scenario: Sockets resolve by name

- **WHEN** the presentation layer mounts a weapon
- **THEN** exactly one node exists for each of `Grip`, `Foregrip`, `Magazine`, and `Muzzle`

#### Scenario: Muzzle marks the shot origin

- **WHEN** presentation spawns a muzzle flash or shot origin
- **THEN** it uses the weapon's `Muzzle` transform

#### Scenario: Magazine marks the reload target

- **WHEN** the reload pose moves the support hand
- **THEN** the `Magazine` socket provides the target and the magazine geometry moves with it

#### Scenario: Socket authoring is validated before export

- **WHEN** a weapon asset is prepared for export
- **THEN** socket names, uniqueness, and positions are checked and previewed with the character pose

## MODIFIED Requirements

### Requirement: Armed rig retains the existing skeleton
The upgraded player asset SHALL keep the existing 31-bone skeleton and its bone names so existing locomotion and death clips remain usable. The rig MUST NOT require retargeting of the current clips and MUST NOT require an added weapon socket bone; weapons attach to a runtime anchor, not to the skeleton.

#### Scenario: Existing clips bind after the upgrade

- **WHEN** the upgraded `player.glb` is loaded with the idle, walk, run, and death clips
- **THEN** every clip binds to its animation targets and plays without missing-bone warnings

#### Scenario: Bone count stays stable

- **WHEN** the asset validation test inspects the upgraded glTF
- **THEN** the skeleton contains the original 31 deform bones and no weapon socket bone

### Requirement: Rig conventions
The exported rig SHALL use a consistent orientation and scale: the character faces `-Z` at rest, feet rest at `Y = 0`, the origin lies between the feet, and the export scale is `1.0`.

#### Scenario: No runtime transform hacks are required

- **WHEN** the corrected model is loaded with safety `scale = 1.0`, `y_offset = 0.0`, and `yaw_offset_deg = 0.0`
- **THEN** the character stands on the arena floor and faces the aim direction without an extra yaw offset

#### Scenario: Facing math matches the asset

- **WHEN** the visual is rotated with `Quat::from_rotation_arc(Vec3::NEG_Z, direction)`
- **THEN** the character's chest points along `direction`

#### Scenario: Weapon space matches the rig

- **WHEN** a weapon is posed at the visual-root anchor
- **THEN** weapon `-Z` forward and `+Y` up align with the character's forward and up without a per-asset offset

### Requirement: Tactical survivor silhouette
The character SHALL read as a low-poly tactical survivor with a vest, cap or helmet, and a small backpack. Gear MUST be merged or parented so it deforms with the correct bones. The weapon is a separate asset and SHALL read as part of the same silhouette when mounted.

#### Scenario: Gear follows animation

- **WHEN** the character walks, aims, and reloads
- **THEN** vest and backpack follow the torso bones without visible detachment

#### Scenario: Mounted weapon reads as held

- **WHEN** the separate weapon is posed and both hands solve to its sockets
- **THEN** the weapon and hands read as one held object at the camera distance

### Requirement: Triangle budget
The upgraded character, including gear but excluding weapons, SHALL remain under 4,000 triangles. Each separate weapon SHALL remain under 1,000 triangles.

#### Scenario: Asset validation reports the budget

- **WHEN** the asset validation test runs on `player.glb` and each weapon glTF
- **THEN** it reports triangle counts for the body and every weapon and fails if either exceeds its budget

### Requirement: Animation clip contract
The player glTF SHALL contain in-place clips for idle, forward walk, backward walk, strafe left, strafe right, and death, using names referenced by the animation catalog. Diagonal anchor clips MAY be added later as catalog entries without Rust changes. Aim and reload clips are optional because aiming and reloading are procedural poses.

#### Scenario: Catalog clip names resolve

- **WHEN** the animation catalog is validated against the player glTF
- **THEN** every required locomotion and death role resolves to exactly one named animation

#### Scenario: Clips are in place

- **WHEN** the locomotion and death clips are exported
- **THEN** they contain no root translation that moves the character away from the origin

#### Scenario: Optional diagonal clip needs no code

- **WHEN** a diagonal clip is added and an anchor entry is added to the catalog
- **THEN** the animation system uses it without a Rust change

#### Scenario: Aim and reload clips are not required

- **WHEN** the player glTF lacks aim and reload clips
- **THEN** the asset still satisfies the clip contract and presentation uses procedural poses

### Requirement: Export contract
`player.glb` SHALL contain the skinned mesh, armature, and character clips in one file, using meters and a `1.0` unit scale. Weapons SHALL be exported as separate glTF binaries and MUST NOT be embedded in `player.glb`.

#### Scenario: Single-file character load

- **WHEN** the game loads `models/player.glb`
- **THEN** the character scene, skeleton, and clips are all present without external texture or mesh dependencies

#### Scenario: Weapons load separately

- **WHEN** the game mounts a weapon
- **THEN** it loads the weapon's own glTF and parents it to the runtime anchor without modifying the character skeleton

#### Scenario: In-place locomotion

- **WHEN** the walk and strafe clips are played
- **THEN** the character's root stays at the origin and movement is driven only by the simulation

### Requirement: Asset work is previewed before export
Changes that alter the character mesh, gear, skeleton, clips, weapons, or sockets SHALL be previewed with rendered images for user approval before the final `player.glb` or weapon glTF is written.

#### Scenario: User approves a rig change

- **WHEN** the rig, gear, or clip edit is ready
- **THEN** a preview render is produced and the export is written only after approval

#### Scenario: User approves a weapon

- **WHEN** a weapon mesh or socket layout is ready
- **THEN** a preview render shows the weapon mounted and held at aim and relaxed poses, and export waits for approval

## REMOVED Requirements

### Requirement: Weapon socket bone
**Reason**: Weapons are independent assets attached to a runtime visual-root anchor; no `WeaponSocket` bone is required in the character skeleton.
**Migration**: Use the runtime weapon anchor and the weapon socket contract instead.

### Requirement: Low-poly rifle
**Reason**: Replaced by the separate weapon asset and socket contract, which covers the rifle and future weapons without embedding a weapon in the character export.
**Migration**: Author each weapon as its own glTF with the required sockets.
