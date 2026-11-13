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
| `sprint_fwd` | `HumanArmature\|Man_Run` | locomotion | present (sprint variant of `walk_fwd`) |
| `death` | `HumanArmature\|Man_Death` | override | present |

Aim, reload, and fire have no clip roles: they are procedural layers driven by `CombatStance`,
the presentation-scoped weapon runtime reload state fed by reload lifecycle events, and
`WeaponFiredMessage`. Sprint uses an optional per-anchor
`sprint_role` (forward anchor → `sprint_fwd` → `Man_Run`) with a smooth gait crossfade; anchors
without a sprint variant fall back to the forward anchor's higher playback multiplier. Missing anchor
clips are dropped once at bind with a warning, and the graph/mask tests do not require the clips to
exist in the GLB.

## Sockets and Weapons

Weapon presentation data lives in the unified roster `game/assets/config/weapon/weapon.ron`, with
simulation stats alongside presentation, separate from the player model. Each weapon points at a
converted GLB under `models/weapons/`: `m16.glb`, `m1911.glb`, `mp5.glb`, `mp7.glb`, and `scar.glb`. The
M16-style assets are low-poly with
simple black materials; the magazine is a separate node under the `Magazine` socket so reload
`mag_drop` moves it.

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

### Model spaces

Converted assets keep the orientation they were exported with. `WeaponDefinition.presentation.model_space`
selects the rotation that maps the model into weapon space, and `model_rotation_deg` adds an optional
extra rotation on top:

| `model_space` | Source axes | Mapping into weapon space |
| :--- | :--- | :--- |
| `Weapon` | `-Z` forward, `+Y` up | identity |
| `ForwardNegYUpZ` | `-Y` forward, `+Z` up | `(x, y, z) -> (-x, z, y)` |

`rifle.glb` is authored in weapon space. The converted `m16`, `m1911`, `mp5`, and `scar` GLBs use
`ForwardNegYUpZ`. Socket overrides are authored in the model's own space because they replace the socket
node's local transform; `model_transform()` then maps them into weapon space together with the rest of
the model.

`model_rotation_deg` adds an extra rotation in weapon space on top of `model_space`. The `m16` profile
uses `model_rotation_deg: (0.0, 180.0, 0.0)` because its GLB's muzzle faced `+Z` (opposite the
character's `-Z` forward) after the `ForwardNegYUpZ` mapping; the extra yaw corrects it to `-Z`. Grip
direction vectors for `m16` were flipped accordingly (X and Z components negated) so the hands aim at
the corrected muzzle end.

### Grip configuration

Grips live in the unified `weapon.ron` roster under each weapon's `presentation.grip` field and are
authored in weapon space:

```ron
grip: Some((
    primary: (
        finger: (0.0, -0.15, -1.0),
        palm: (-1.0, 0.15, 0.0),
        elbow: (0.35, -1.0, 0.25),
        fingers: Some((middle_hand_deg: 22.0, fingers_deg: 45.0, thumb1_deg: 18.0, thumb2_deg: 22.0)),
    ),
    support: Some((
        finger: (0.55, 0.10, -1.0),
        palm: (0.25, 1.0, 0.0),
        elbow: (-0.35, -1.0, 0.25),
    )),
)),
```

- `position` is optional. When omitted, the palm sits at the socket position plus a palm-side offset.
- `finger` points from the wrist to the knuckles and `palm` is the direction the palm presses into the
  grip. `elbow` is a weapon-space bend hint.
- `wrist_deg` trims the solved hand rotation in the hand bone's own frame; `align_deg` tilts the hand in
  weapon space after the palm frame.
- `fingers` sets the per-hand curl; omitted values fall back to `IkConfig::right_finger_curl` or
  `left_finger_curl`.
- Every field is optional and merges field by field over the socket-derived grip, so a new gun only needs
  a catalog entry. `support: None` makes the weapon one-handed.
- `min_support_separation` (default `0.08`) keeps a foregrip socket that is too close to the grip from
  producing a bogus support hand.

The `m16` profile flips its grip direction vectors (`finger`, `palm`, `elbow`: X and Z components
negated) because `model_rotation_deg: (0.0, 180.0, 0.0)` rotates the weapon 180° around Y in weapon
space. All other weapon profiles use the unflipped vectors shown above.

### Hand IK order

Each frame runs, in order: animation evaluation (`Drive`), weapon pose and idle relaxation (`Weapon`),
then both arm solves and finger curls (`Ik`). Idle relaxation therefore runs before IK, so the arm solve
targets the relaxed torso instead of fighting a correction applied afterwards. The right chain solves to
the primary grip and the left chain to the support grip; unreachable targets are clamped to arm reach and
the shoulders settle within `settle_limit_deg`.

Run with `cargo run --features dev,dev-ik-gizmo` to draw the resolved hand frames, wrist trims, elbow
poles, and solved arm chains for calibration. The gizmo is dev-only and does not exist in production
builds.

Weapons are mounted under a chest-height anchor on `PlayerVisualRoot`; the simulation remains
authoritative for projectile origin and presentation sockets are cosmetic.

`Body`, `Bone`, and the pole targets are left unmasked.

## Remaining Asset Limitations

- Backward walk and left/right strafe clips are authored (`Man_WalkBack`, `Man_StrafeL`, `Man_StrafeR`) on the existing 31-bone rig; strafe is a procedural side-shuffle with upper body/arm motion carried over from `Man_Walk`.
- `models/weapons/rifle.glb` ships with `Grip`, `Foregrip`, `Magazine`, `Muzzle` sockets (grip at origin, `-Z` forward, `+Y` up, meters). Hand orientation comes from the grip configuration and the per-hand wrist trims rather than per-socket rotations.
- Runtime safety scale/offset/yaw adjustments remain in `visuals.ron` until the corrected origin/scale asset is approved.
- Finger closure is procedural: `IkConfig` supplies a default curl per hand and a weapon can override it per grip. No jump, crouch, or hit-react clip exists yet; the override layer is reserved for death only.

## Revision notes (2026-11-13)

- Fixed locomotion weight normalization so anchor weights sum to `1 - idle`.
- Idle relaxation now uses `AnimCatalog.locomotion.move_speed_threshold`.
- Weapon GLB loads with no scenes (or no profile) are marked failed instead of retrying forever.
- Corrected `mp5`, `mp7`, and `scar` grip authoring so primary=back grip, support=front grip.
- Moved non-M16 weapon pose offsets from positive to negative Z so they sit in front of the player.

## Revision notes (2026-11-14)

- Fixed the `m16` `Grip` socket override in `weapon/weapon.ron`: the X offset was `10.0`, which
  placed the right-hand IK grip target ~14 m from the weapon; the arm solver clamped to max reach
  and dragged the hand down across the body. Set to `(0.0, -0.060, -0.030)`, matching the other
  weapon profiles. Verified live via BRP: the `Grip` socket now resolves at the weapon origin.
  Open follow-up (not yet fixed): in the relaxed (`aim = 0`) pose the gun sits at hip height, yawed
  90° across the body (`relaxed rotation_deg = (-32, 90, 0)`); measured reach from the shoulders is
  ~0.56 m, while `Shoulder.R -> Grip` is ~0.60 m and `Shoulder.L -> Foregrip` is ~0.81 m, so the
  hands land short of the sockets (Palm.R ~0.18 m, Palm.L ~0.5 m off). Whether the relaxed yaw/pose
  or the reach shortfall should change is a design call.

## Validation

`tools/blender/inspect_glb.py` reports nodes, bones, triangles, clip names, and can require
specific bones, nodes, or clips. Use it before and after the rig upgrade.
