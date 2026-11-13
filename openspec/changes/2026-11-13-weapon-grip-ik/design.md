## Context

Weapon presentation mounts a model under a chest-height anchor and solves both arms onto the weapon
sockets. The previous IK used one `hand_offset_deg` for both hands plus catalog-wide pole vectors, so the
right palm, left palm, and elbow bend could not be tuned independently. Converted GLBs also arrive in
different orientations, and finger curls were catalog-wide rather than per grip.

## Goals / Non-Goals

**Goals:**

- A reusable, data-driven grip: per-hand position, finger direction, palm direction, elbow hint, finger
  curl, wrist trim, and weapon-space alignment.
- Correct two-handed grips for the current roster and automatic, sensible defaults for any new weapon.
- One-handed weapons via `support: None`.
- Model-space conversion so converted assets need no code-side rotation hacks.
- Calibration support through a dev-only gizmo.

**Non-Goals:**

- Authoring final hand animation clips or IK for non-weapon interactions.
- Replacing the simulation's authoritative muzzle/projectile math.
- Adding new weapon art or re-exporting sockets.

## Decisions

### D1. Authored grip merges field by field over a socket-derived grip

Each `GripHand` field is optional. The resolver derives a base hand from the weapon's socket frames and
replaces only the fields the author sets. This lets a weapon tune its palm direction without restating its
finger direction, and lets a brand-new weapon work with no grip entry at all.

### D2. Grip values are authored in weapon space

`+Y` up, `-Z` toward the muzzle, `+X` toward the weapon's right, matching character axes. Socket frames
are converted into that space once, so grip targets and elbow poles share one basis regardless of how the
model was exported.

### D3. Weapon basis comes from the grip socket, with the muzzle as a forward hint

The grip socket rotation supplies the base basis. When both grip and muzzle exist, the muzzle-minus-grip
direction overrides forward so a model with identity socket rotations still yields a correct forward axis.
The basis is re-orthogonalised to stay finite and right-handed.

### D4. Hand frames replace Euler offsets

A hand target is a `HandFrame` built from `finger` and `palm` directions (`+Y` wrist-to-knuckles, `+Z` out
of the back of the hand, `+X` completing the frame). `align_deg` rotates the frame in weapon space;
`wrist_deg` trims the solved hand rotation in the hand bone's own frame. This is more stable than a shared
Euler offset because each hand can mirror the other naturally.

### D5. Elbow poles are part of the grip

`elbow` is a weapon-space direction, transformed into character space with the weapon. This removes
`right_pole`/`left_pole` from `IkConfig` and lets a pistol grip and a foregrip bend differently.

### D6. Finger curls live with the grip, with catalog fallbacks

`FingerCurl` moved into the grip module and stores one scalar per bone (the bones fold around local X).
A weapon can override the curl per hand; otherwise `IkConfig::right_finger_curl`/`left_finger_curl` apply.

### D7. Model space is a profile field

`model_space: Weapon | ForwardNegYUpZ` selects the mapping into weapon space. `ForwardNegYUpZ` maps
`(x, y, z)` to `(-x, z, y)` (Euler `(90, 180, 0)`). `model_rotation_deg` adds an optional extra rotation and
`model_scale` stays uniform. Socket overrides are model-space transforms because they replace the socket
node's local transform.

### D8. Idle relaxation runs before IK

Relaxing the torso after IK moved the arms away from their targets. Moving the relaxation into
`PlayerAnimSet::Weapon` before `PlayerAnimSet::Ik` lets the solve target the relaxed body.

### D9. Dev-only gizmo behind `dev-ik-gizmo`

`IkDebug` captures targets, poles, and chains during the solve. `draw_ik_gizmo` runs after the solve and
draws hand frames, wrist trims, poles, and arm lines. The resource and system are compiled only with the
feature, so production builds contain no debug code.

## Risks / Trade-offs

- **Calibration is visual** -> grip values need a pass in-game with the gizmo; defaults are plausible but
  not final. Mitigation: gizmo plus per-field overrides make tuning local.
- **Socket overrides are model-space** -> authors must remember the model's axes. Mitigation: documented
  model-space table and the `ForwardNegYUpZ` mapping.
- **Per-hand curl can over-close fingers** -> validation caps curls at 120 degrees and the grip merge keeps
  catalog fallbacks.
- **One-handed fallback** -> a missing foregrip with an authored support hand still uses a safe support
  position in front of the grip rather than dropping the hand.

## Migration Plan

1. Add the grip data model and resolver.
2. Add socket-frame conversion and the weapon basis.
3. Replace IK targets with hand frames, wrist trims, and per-hand poles.
4. Add model-space mapping and socket overrides.
5. Move finger curls into the grip config and apply them after the arm solve.
6. Reorder idle relaxation before IK.
7. Update `weapons.ron` and `animation.ron`.
8. Add the dev gizmo, tests, and docs.

## Open Questions

- (none)
