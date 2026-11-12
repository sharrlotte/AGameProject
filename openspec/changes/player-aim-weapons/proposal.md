## Why

The current animation stack keeps the lower body aligned to movement through cone rules, so the character
does not consistently face the cursor, diagonals collapse into a four-clip blend, sprint steals facing from
aim, and movement input is mapped to world axes. The weapon is not layered at all: `aim` and `reload` are
whole-upper-body clips, the rifle is expected to ride a hand socket, and no weapon glTF or socket exists yet
(raw FBX gun meshes were added to the workspace on 2026-11-12 but are not integrated).

The chosen model is **always-aim**: the visual root always faces the cursor, and locomotion is selected by
movement direction measured in the character's aim space. That removes cone hysteresis entirely and makes
directional locomotion a pure data problem. Weapon handling then becomes procedural pose on top of
locomotion, which removes per-weapon upper-body clips from the asset budget.

This change **supersedes `player-upper-lower-rotation`**. The cone-based facing, idle catch-up turn, and
procedural spine twist implemented there are removed rather than extended.

## What Changes

- **Always-aim facing (Model A)**: `PlayerVisualRoot` yaw follows the horizontal aim direction with
  shortest-arc smoothing. Sprint no longer takes facing away from aim. Movement never drives body yaw.
- **Data-driven locomotion anchors**: replace forward/back/left/right hardcoding with a catalog list of
  angular anchors in character-local space. Ship the four cardinal clips now; diagonal clips can be added
  later without Rust changes. Weights come from the two anchors bracketing the local movement angle.
- **Foot-phase continuity**: when the dominant anchor changes, the incoming clip seeks to the outgoing
  clip's normalized phase before blending, so feet do not pop.
- **Camera-relative movement input**: build the input basis from the camera's yaw instead of world X/Z, so W
  is always screen-up. The fixed camera makes this a no-op today; it removes a future follow-camera rewrite.
- **Procedural weapon pose**: a runtime weapon anchor in visual-root space holds a separate weapon glTF.
  Each weapon exposes `Grip`, `Foregrip`, `Magazine`, and `Muzzle` sockets. Aim and relaxed poses are
  offsets on that anchor, not animation clips.
- **Dual-hand IK**: both arms IK to `Grip` and `Foregrip` whenever a weapon is mounted and the player is
  alive, with configurable shoulder/clavicle settling. IK replaces arm motion from locomotion clips.
- **Procedural reactions**: recoil from `WeaponFiredMessage`, swap from `WeaponSwappedMessage`, and reload
  from `WeaponInventory::reload_progress()` drive anchor offsets and IK targets. No per-weapon upper-body
  clips are required.
- **Simplified graph**: base locomotion, upper-body additive action, and full-body death override. The aim
  clip layer is removed; the action layer becomes optional.
- **Remove obsolete code**: cone facing, `MoveZone`, idle catch-up, `TwistConfig`, `anim/twist.rs`, and the
  hand-socket rifle contract.
- **Split animation and weapon data**: `animation.ron` keeps locomotion, masks, IK chains, and smoothing;
  a new hot-reloadable weapon catalog holds per-weapon models, sockets, poses, recoil, and reload timing.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `player-animation`: always-aim facing, anchor-based directional blending with phase continuity,
  camera-relative input basis, procedural weapon pose and dual-hand IK, procedural recoil/reload/swap,
  simplified layer stack, and updated catalog contract.
- `player-rig`: weapons become separate glTF assets with a named socket contract instead of a rifle baked
  into `player.glb` behind a `WeaponSocket` bone; clip contract drops mandatory aim/reload clips.

## Impact

- **Code (`crates/presentation`)**: `anim/facing.rs` (rewrite), `anim/director.rs`, `anim/graph.rs`,
  `anim/catalog.rs`, `anim/rig.rs`, `anim/ik.rs`, `anim/bind.rs`, `anim/mod.rs`; remove `anim/twist.rs`;
  new weapon pose/socket module; `input/system.rs` if camera-relative input is approved.
- **Config (`game/assets/config/player`)**: `animation.ron` (anchors, aim pose, IK chains), new weapon
  catalog asset.
- **Assets (`game/assets/models`)**: `player.glb` clip set (backward and strafe anchors, aim/reload clips
  no longer required); new `game/assets/models/weapons/*.glb` with sockets.
- **Documentation (`docs`)**: `01-combat-movement-and-weapons.md`, `07-technical-architecture-save-and-roadmap.md`,
  `09-player-asset-baseline.md`, plus a weapon rig/animation contract document.
- **OpenSpec**: archive `player-upper-lower-rotation` as superseded, not completed.

Simulation movement, aim projection, weapon firing, and reload timing are unchanged.
