# Player Asset Baseline

Recorded before the layered animation work, for comparison after the rig upgrade.

Source: `game/assets/models/player.glb`

## Current Asset

| Property | Value |
| :--- | :--- |
| Nodes | 33 |
| Deform bones | 31 |
| Triangles | 1,852 |
| Materials | Low-poly, untextured / simple |
| Forward axis at rest | Runtime adds `yaw_offset_deg = 180` |
| Runtime scale | `0.37` |
| Runtime Y offset | `-0.9` |

## Current Bones

```
Bone                  Foot.L                Body
Hips                  Abdomen               Torso
Neck                  Head                  Shoulder.L
UpperArm.L            LowerArm.L            Palm.L
MiddleHand.L          Fingers.L             Thumb1.L
Thumb2.L              Shoulder.R            UpperArm.R
LowerArm.R            Palm.R                MiddleHand.R
Thumb1.R              Thumb2.R              Fingers.R
UpperLeg.L            LowerLeg.L            UpperLeg.R
LowerLeg.R            PoleTarget.L          Foot.R
PoleTarget.R
```

## Current Clips

```
HumanArmature|Man_Clapping
HumanArmature|Man_Death
HumanArmature|Man_Idle
HumanArmature|Man_Jump
HumanArmature|Man_Punch
HumanArmature|Man_Run
HumanArmature|Man_RunningJump
HumanArmature|Man_Sitting
HumanArmature|Man_Standing
HumanArmature|Man_StrafeL
HumanArmature|Man_StrafeR
HumanArmature|Man_SwordSlash
HumanArmature|Man_Walk
HumanArmature|Man_WalkBack
```

There is no aim, reload, fire, strafe, weapon socket, or rifle.

## Runtime Mask Groups

The catalog in `game/assets/config/player/animation.ron` uses explicit bone lists because `Hips` is the
ancestor of the whole spine. These are the masks registered against live `AnimationTargetId`s:

Lower body (locomotion layer):

```
Body, Bone, Hips
UpperLeg.L, LowerLeg.L, Foot.L
UpperLeg.R, LowerLeg.R, Foot.R
```

Upper body (action layer):

```
Abdomen, Torso, Neck, Head
Shoulder.L, UpperArm.L, LowerArm.L, Palm.L, MiddleHand.L, Fingers.L, Thumb1.L, Thumb2.L
Shoulder.R, UpperArm.R, LowerArm.R, Palm.R, MiddleHand.R, Fingers.R, Thumb1.R, Thumb2.R
```

`PoleTarget.L` and `PoleTarget.R` are left unmasked so neither layer owns them.

## Runtime Clip Roles

| Role | Current glTF clip | Layer | Status |
| :--- | :--- | :--- | :--- |
| `idle` | `HumanArmature\|Man_Idle` | locomotion | present |
| `walk_fwd` | `HumanArmature\|Man_Walk` | locomotion | present |
| `walk_back` | `HumanArmature\|Man_WalkBack` | locomotion | present |
| `strafe_l` | `HumanArmature\|Man_StrafeL` | locomotion | present |
| `strafe_r` | `HumanArmature\|Man_StrafeR` | locomotion | present |
| `death` | `HumanArmature\|Man_Death` | override | present |

Aim, reload, and fire have no clip roles: they are procedural layers driven by `CombatStance`,
`WeaponInventory::reload_progress()`, and `WeaponFiredMessage`. Sprint uses the forward anchor with a
higher playback multiplier instead of a separate clip role. Missing anchor clips are dropped once at
bind with a warning, and the graph/mask tests do not require the clips to exist in the GLB.

## Sockets and Weapons

Weapon presentation data lives in `game/assets/config/player/weapons.ron`, separate from the player
model. The `assault_rifle` profile points at `models/weapons/rifle.glb`, converted from
`game/assets/models/weapons/guns/M16.fbx` (M16-style, low-poly, 1,750 triangles, 8 materials, all
shades of black: `Mat_Receiver`/`Mat_Barrel` dark gray metal, `Mat_Handguard`/`Mat_Stock`/
`Mat_Grip` matte dark gray polymer, `Mat_Magazine`/`Mat_Sight` near-black, `Mat_Detail` dark gray).
The magazine is a separate
node under the `Magazine` socket so reload `mag_drop` moves it.

Socket contract for every weapon glTF:

| Name | Purpose | Fallback |
| :--- | :--- | :--- |
| `Grip` | right-hand IK target and weapon origin | weapon root |
| `Foregrip` | left support-hand IK target | weapon root |
| `Magazine` | left-hand target during reload and magazine drop animation | foregrip |
| `Muzzle` | future muzzle flash/tracer origin | none |

Required orientation: `-Z` forward, `+Y` up, meters, grip at the origin. Socket names are configurable
per profile, so a re-export can rename them without code changes. Missing sockets are reported once and
the affected hand falls back as listed above.

Weapons are mounted under a chest-height anchor on `PlayerVisualRoot`; the simulation remains
authoritative for projectile origin and presentation sockets are cosmetic.

`Body`, `Bone`, and the pole targets are left unmasked.

## Remaining Asset Limitations

- Backward walk and left/right strafe clips are authored (`Man_WalkBack`, `Man_StrafeL`, `Man_StrafeR`) on the existing 31-bone rig; strafe is a procedural side-shuffle with upper body/arm motion carried over from `Man_Walk`.
- `models/weapons/rifle.glb` ships with `Grip`, `Foregrip`, `Magazine`, `Muzzle` sockets (grip at origin, `-Z` forward, `+Y` up, meters). Socket rotations are identity; hand orientation may be tuned via `hand_offset_deg` or per-socket rotations during runtime verification.
- Runtime safety scale/offset/yaw adjustments remain in `visuals.ron` until the corrected origin/scale asset is approved.
- Finger closure on the grip is not authored: finger bones follow locomotion clips, so the hand does not fully close around the weapon. No jump, crouch, or hit-react clip exists yet; the override layer is reserved for death only.

## Validation

`tools/blender/inspect_glb.py` reports nodes, bones, triangles, clip names, and can require
specific bones, nodes, or clips. Use it before and after the rig upgrade.
