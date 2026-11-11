# Player Asset Baseline

Recorded before the layered animation work, for comparison after the rig upgrade.

Source: `game/core/assets/models/player.glb`

## Current Asset

| Property | Value |
| :--- | :--- |
| Nodes | 45 |
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
HumanArmature|Man_SwordSlash
HumanArmature|Man_Walk
```

There is no aim, reload, fire, strafe, weapon socket, or rifle.

## Runtime Mask Groups

The catalog in `game/core/assets/config/animation.ron` uses explicit bone lists because `Hips` is the
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
| `walk_back` | `HumanArmature\|Man_Walk` | locomotion | placeholder |
| `sprint` | `HumanArmature\|Man_Run` | locomotion | present |
| `strafe_l` | `HumanArmature\|Man_StrafeL` | locomotion | missing until authored |
| `strafe_r` | `HumanArmature\|Man_StrafeR` | locomotion | missing until authored |
| `aim` | `HumanArmature\|Man_Aim` | action | missing until authored |
| `reload` | `HumanArmature\|Man_Reload` | action | missing until authored |
| `death` | `HumanArmature\|Man_Death` | override | present |

Missing clips are logged at graph build and the director degrades gracefully to relaxed arms. The
graph and mask tests do not require the clips to exist in the GLB.

## Sockets and Rifle

| Name | Parent | Purpose | Status |
| :--- | :--- | :--- | :--- |
| `WeaponSocket` | right hand | attaches the rifle | planned |
| `Foregrip` | rifle | left support-hand IK target | planned |
| `MuzzleSocket` | rifle | future muzzle flash/tracer origin | planned |

The simulation remains authoritative for projectile origin; presentation sockets are cosmetic.

`Body`, `Bone`, and the pole targets are left unmasked.

## Remaining Asset Limitations

- The current GLB still lacks `strafe_l`, `strafe_r`, `aim`, and `reload` clips; the director degrades to relaxed arms and forward locomotion until they are authored.
- `walk_back` currently reuses `Man_Walk`, so backward locomotion reads as a forward clip until a dedicated clip is authored.
- Runtime safety scale/offset/yaw hacks remain in `visuals.ron` until the corrected origin/scale asset is approved.
- No jump, crouch, or hit-react clip exists yet; the override layer is reserved for death only.

## Validation

`tools/blender/inspect_glb.py` reports nodes, bones, triangles, clip names, and can require
specific bones, nodes, or clips. Use it before and after the rig upgrade.
