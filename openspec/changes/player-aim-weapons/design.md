## Context

`crates/presentation/src/anim` currently implements a split-facing model: `facing.rs` tracks lower-body yaw,
upper-body aim yaw, a smoothed spine twist, a `MoveZone` cone, and an idle catch-up turn; `director.rs`
projects velocity onto the lower-body yaw and blends four fixed roles; `twist.rs` distributes residual yaw
across `Hips`, `Abdomen`, and `Torso`. `graph.rs` builds a masked lower/upper graph where `aim` and `reload`
are whole-upper-body clips. `rig.rs` expects a `Foregrip` bone and `ik.rs` only solves the left arm.

The user selected **Model A**: the body always faces the cursor and locomotion is chosen from movement
relative to aim. That makes the cone state, idle catch-up, and spine twist unnecessary: with the root
facing aim, there is no residual twist to distribute. It also makes `aim` and `reload` clips unnecessary as
pose sources, because a weapon pose plus two-arm IK describes the upper body directly.

`game/assets/models/player.glb` currently has 11 clips (`Man_Idle`, `Man_Walk`, `Man_Run`, `Man_Death`,
plus extra idle variants), no backward or strafe clip, no aim or reload clip, and no weapon glTF or sockets.
Raw FBX gun meshes exist in `game/assets/models/weapons/guns/` but are not integrated. Work is therefore
designed to run with fallbacks until assets land.

## Goals / Non-Goals

**Goals:**

- Body yaw continuously follows the cursor, including while sprinting and strafing.
- Locomotion is selected by movement direction in aim space, with a data-driven anchor set that supports
  four cardinal clips today and eight directions later without Rust changes.
- Smooth analog blending between adjacent anchors, preserved foot phase, speed-scaled playback.
- Weapon pose is procedural and weapon-independent: one anchor, per-weapon sockets, two-arm IK, shoulder
  settle, recoil, reload, and swap derived from existing simulation messages/progress.
- Remove cone/twist machinery and mandatory aim/reload clips.
- Keep animation data hot-reloadable and systems scheduled only in gameplay states.

**Non-Goals:**

- Changing simulation movement speed, aim projection, fire rate, reload duration, or weapon switching rules.
- Authoring per-weapon upper-body animation clips.
- Enemy animation changes.
- Predicting or compensating network latency.
- In-place turn clips (deferred; the smoothing path leaves room for them).

## Decisions

### D1. Always-aim body orientation (Model A)

`PlayerVisualRoot.rotation` is a yaw-only rotation toward `AimDirection`. Movement never writes body yaw.
Sprint does not override aim. When aim is zero/unavailable, the previous yaw is retained. On death, the
visual root stops tracking aim so the death clip plays in place.

Why: this is the user-selected model. It also removes the largest source of state and jitter (cone
hysteresis) and makes local movement angles well-defined every frame.

### D2. Aim yaw smoothing with shortest arc

Keep a `PlayerFacing { yaw: f32, target: f32 }` (radians, yaw only). Each frame:

```text
delta = wrap_pi(aim_yaw - yaw)
alpha = 1 - exp(-dt / max(blend_secs, epsilon))
yaw  += delta * alpha
```

`blend_secs` comes from the catalog (`facing.blend_secs`, default ~0.12 s). The exponential form is
frame-rate independent and cannot overshoot. A large 180-degree flick rotates over roughly 3x
`blend_secs`, which reads as a deliberate turn rather than a snap.

Alternative considered: slerp on `Quat`. Rejected because yaw-only float math avoids normalization drift,
makes wrap tests trivial, and is all the rig needs.

### D3. Locomotion angle is measured in aim space

With the asset facing `-Z` and local right `+X`:

```text
local   = rotate_y(-facing.yaw) * normalized(horizontal_velocity)
angle   = atan2(local.x, -local.z)        // 0 = forward, +pi/2 = right, pi = back
```

Anchors are catalog entries `(role, angle_deg)` using the same convention. This is the only place the
convention lives; a different asset convention changes the projection, not the anchor data.

### D4. Weighting: bracket the local angle with two anchors

At bind time, resolve each anchor's clip node and duration, drop anchors whose clip is missing (with one
warning), and sort the survivors by angle. Per frame:

1. Wrap the local angle and anchor angles to `[0, 2pi)`.
2. Find the first anchor strictly greater than the local angle; its predecessor (circular) is the lower
   bracket. If the angle equals an anchor exactly, that anchor gets full weight.
3. `span = (next - prev) mod 2pi`, `t = ((angle - prev) mod 2pi) / span`.
4. Weights are `1 - t` and `t`; every other anchor is zero.

This gives partition of unity, continuity across the `+/-pi` seam, exact single-clip poses on anchors, and
50/50 blends on diagonals when only cardinals exist. With eight anchors, a diagonal clip receives full
weight at its angle.

If only one anchor remains, it gets full weight. If none remain, the idle clip plays and a warning is
logged once. Anchors are capped at 8; weights live in a fixed-size array inside `DirectorState`, so no
per-frame allocation or hash lookup occurs.

Alternative considered: per-anchor window functions. Rejected because overlapping windows need
normalization, produce three-clip blends with uneven spacing, and make phase/dominant-anchor selection
ambiguous. Bracketing always blends at most two clips.

### D5. A single normalized locomotion phase keeps feet aligned

`DirectorState` owns one `phase: f32` in `[0, 1)` and advances it by
`dt * playback_scale / duration(dominant_anchor)`. On a dominant-anchor change, the incoming clip seeks to
`phase * its_duration` before its weight rises. The dominant anchor is the highest-weight anchor. This
prevents the classic diagonal foot pop where two clips start at arbitrary phases.

Per-frame playback speed is `clamp(speed / walk_speed_ref, speed_scale_min, speed_scale_max)`, multiplied by
`sprint_playback_speed` while sprinting. Idle never advances the phase.

Phase is data-independent and survives graph hot reload by resetting to zero together with the graph.

### D6. Camera-relative movement input (approved)

`input/system.rs` currently maps `W/S/A/D` to world `-Z/+Z/-X/+X`. This change projects the camera's forward
onto XZ, derives right = `forward x up`, and composes `forward * y + right * x`. When no camera exists it
falls back to world axes. Because the camera is currently fixed at `(0, 22, 12.5)`, behaviour is unchanged
today; the change only future-proofs follow/rotating cameras.

### D7. Runtime weapon anchor instead of a hand socket

Add a `WeaponAnchor` entity parented to `PlayerVisualRoot` at a configurable offset and rotation near chest
height. A mounted weapon is a separate glTF scene parented to that anchor. Its local transform is composed
from:

```text
anchor = base_offset * relaxed_or_aim_pose * sway * recoil * reload_offset * swap_offset
```

Aiming raises the weapon to eye line; relaxed lowers it across the chest. Both poses are catalog offsets,
not clips. The weapon is never parented to the hand, so recoil and reload can move the weapon while IK
keeps the hands attached.

Why: the user chose procedural weapon handling and separate weapons. This also means `player.glb` contains
no rifle, and adding a weapon is a data + asset change only.

### D8. Socket contract on weapon assets

Each weapon glTF exposes exactly one node per socket: `Grip`, `Foregrip`, `Magazine`, `Muzzle`. Socket
transforms are authored in weapon space, weapon forward is `-Z`, up is `+Y`, units are meters, origin is at
the grip. Sockets are resolved once at mount and cached as entity IDs; missing sockets downgrade to the
weapon root and log a warning. `Muzzle` is used for flash/impact origin; `Magazine` is the reload hand
target and the mesh that drops during reload.

Optional per-weapon socket overrides may be supplied in the catalog for assets that cannot be re-exported,
but authored sockets are the contract.

### D9. Dual-hand IK and shoulder settle

When a weapon is mounted and the player is alive, IK runs for both arms:

1. Compose chest and shoulder transforms in visual-root space.
2. Solve the right chain (`UpperArm.R`, `LowerArm.R`, `Palm.R`) to `Grip`, then align the palm to the grip
   frame.
3. Solve the left chain (`UpperArm.L`, `LowerArm.L`, `Palm.L`) to `Foregrip`, then align to the foregrip
   frame.
4. Rotate each shoulder/clavicle by `shoulder_settle *` the yaw/pitch needed by its arm, clamped to a
   catalog limit, before the arm solve. `shoulder_settle` defaults to `0.35`.

The existing analytic two-bone solver is generalized to take a chain, a target transform, a pole vector,
and a hand alignment mode. Targets are clamped to reachable distance, so an unreachable socket extends the
arm without inverting joints. IK writes bone rotations after `AnimationSystems`; it never writes simulation
state. When no weapon is mounted, IK is skipped and locomotion arm motion plays normally.

Why right-then-left: the right hand defines the weapon's presented orientation; solving the support hand
second avoids the support hand pulling the weapon pose around.

### D10. Procedural reactions from existing messages/progress

- **Recoil**: on `WeaponFiredMessage` where `shooter` is the player, add a pitch kick and backward offset
  to a decaying `WeaponKick` state (critically damped, catalog `recover_secs`). Optional small additive
  chest pitch is applied before IK so the body absorbs the shot. Fire rate does not stack without bound;
  impulses clamp to a max.
- **Reload**: while `WeaponInventory::reload_progress()` is in `(0, 1)`, the left hand targets `Magazine`,
  the magazine child translates down and back, and the weapon tilts by `tilt_deg`. The hand returns to
  `Foregrip` over the last portion. Progress, not clip time, drives the pose, so reload-speed upgrades stay
  in sync. Cancellation blends the hand back to the foregrip. Reload is fully procedural; no reload clip is
  authored.
- **Swap**: on `WeaponSwappedMessage`, start a short lower/raise timeline. The old weapon hides, the new
  weapon appears parented to the anchor, sockets re-resolve, and both hands blend from the lowered pose to
  the current aim/relaxed pose. No clip is required.

Why: these events already exist and are unused by animation; using them keeps presentation decoupled from
combat logic. It also removes three clip roles from the asset budget.

### D11. Simplified layered graph and schedule

Graph layers become:

- base locomotion blend (full body, anchors),
- upper-body additive action (optional clips such as hit reactions; masked to `upper`),
- full-body death override.

Aim is no longer a clip layer. Arms are overridden by IK after graph evaluation. Masks, mask registration,
and death override semantics stay as they are.

`PostUpdate` ordering, all before `TransformSystems::Propagate`:

```text
drive_player_animation      // weights, phase/seek, speed
AnimationSystems            // evaluate graph
pose_player_weapon          // pose, sway, recoil, reload, swap offsets
solve_player_arms           // shoulder settle, right arm, left arm
sync_player_visual_root     // write smoothed aim yaw
```

`pose_player_weapon` and `solve_player_arms` compose transforms in visual-root space from `PlayerFacing.yaw`
rather than relying on this frame's propagated root transform, so there is no one-frame lag between body
yaw and hand targets.

Systems stay gated to `Playing` and `GameOver`. Death suppresses weapon pose and IK in the same frame the
override takes weight.

### D12. Split catalogs and hot reload

`animation.ron` gains:

```text
facing:   (blend_secs)
locomotion: (idle_role, move_speed_threshold, anchors: [(role, angle_deg)], speed settings, phase_sync)
aim:      (blend_secs, aim_height)
ik:       (right_chain, left_chain, shoulder_settle, settle_limit_deg, poles, hand_align)
masks:    (unchanged)
clips:    (locomotion + optional action + death; aim/reload no longer required)
```

A new hot-reloadable `weapons.ron` holds per-weapon `id`, `model` path, socket names, relaxed/aim offsets,
recoil, reload, swap, and optional socket overrides. Splitting keeps the animation catalog stable as the
weapon roster grows.

Hot reload rebuilds the graph (animation), respawns the mounted weapon and re-resolves sockets (weapons),
clears phase and kick state, and keeps the player entity alive. Graph handles are dropped exactly once.

### D13. Removal plan for the superseded implementation

- Delete cone/zone/idle-catch-up/twist state from `facing.rs`; keep a yaw-only facing component.
- Delete `anim/twist.rs`, its schedule slot, `TwistConfig`, and the `twist` config section.
- Replace `Weights::from_local` forward/back/left/right with anchor weights.
- Remove aim/reload clip nodes and required roles; keep optional action clips if authored.
- Remove `WeaponSocket` bone assumption from `rig.rs`; resolve weapon sockets from the weapon glTF.
- Delete the old change's remaining tasks (GLB export, twist docs/tests) when it is archived as
  superseded rather than completed; the archived copy keeps them as an obsolete record only.

## Resolved Decisions

All open questions were answered on 2026-11-12 and are now binding for this change.

- **Reload**: procedural timeline from `reload_progress()`. No reload clip is authored.
- **Weapon anchor parent**: `PlayerVisualRoot`. Chest parenting is rejected for this change.
- **Shoulder settle**: enabled with default weight `0.35` and a catalog clamp.
- **Camera-relative input**: implemented in this change, with a world-axis fallback.
- **Superseded change**: `player-upper-lower-rotation` is archived now as superseded, not completed.
- **Weapon data location**: separate `weapons.ron` catalog.

## Risks / Trade-offs

- **IK overrides locomotion arm swing** → Relaxed pose plus shoulder settle preserves some body motion;
  unarmed states skip IK. If arms look stiff, add a low-weight arm additive clip later without changing
  the pose pipeline.
- **Weapon welded to root looks rigid** → Sway, recoil, reload tilt, and chest additive pitch provide
  motion; chest parenting remains a future option but is out of scope.
- **Phase sync drift across clips with different durations** → One normalized clock plus seek on dominant
  change; catalog may enforce equal reference durations for anchor clips.
- **Procedural reload can read as flat** → Magazine drop, tilt, and hand travel; preview before shipping.
- **Uneven or missing anchors** → Bind-time validation rejects duplicate/out-of-range anchors and drops
  missing clips with a single warning; bracketing guarantees a valid two-anchor blend.
- **Asset work blocks visual verification** → Code paths degrade with warnings (missing clips, missing
  sockets) so tests and integration can proceed before assets land.
- **Two IK chains add per-frame cost** → Chains and sockets are cached; the solver is closed-form and runs
  only in gameplay states.

## Migration Plan

1. Extend the catalog with facing, anchors, aim, IK, and weapon data; add `weapons.ron`; parse/validation
   tests first.
2. Rewrite facing to yaw-only always-aim and replace director weights with anchor bracketing plus phase.
3. Add weapon anchor, socket resolution, pose offsets, recoil/reload/swap, and dual-arm IK behind the
   existing graph; keep warnings for missing assets.
4. Simplify the graph and remove aim/reload nodes, cone state, twist module, and obsolete config.
5. Apply camera-relative input.
6. Author directional clips and weapon assets with preview approval; wire real socket names.
7. Update docs, run targeted presentation tests, archive the superseded change.
