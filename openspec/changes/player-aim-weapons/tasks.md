## 1. Catalog, Config & Tests

- [x] 1.1 Replace `TwistConfig` in `crates/presentation/src/anim/catalog.rs` with `FacingConfig` (`blend_secs`), `LocomotionConfig` (`idle_role`, `move_speed_threshold`, `anchors: Vec<Anchor>`, `phase_sync`, speed settings), `AimConfig` (`blend_secs`, `aim_height`), and an IK config covering both arm chains, poles, hand alignment, and shoulder settle.
- [x] 1.2 Add `WeaponCatalog` (per-weapon `id`, `model`, socket names, relaxed/aim offsets, recoil, reload, swap, optional socket overrides) as a separate hot-reloadable RON asset.
- [x] 1.3 Update `game/assets/config/player/animation.ron` with the four cardinal anchors (`walk_fwd` 0, `strafe_r` 90, `walk_back` 180, `strafe_l` -90), facing, aim, and dual-arm IK settings; remove the `twist` section.
- [x] 1.4 Add `game/assets/config/player/weapons.ron` with the initial rifle profile and socket names.
- [x] 1.5 Add unit tests for anchor validation (unique angles, range, at least one anchor), catalog parsing, weapon profile parsing, and defaults.

## 2. Always-Aim Facing

- [x] 2.1 Rewrite `crates/presentation/src/anim/facing.rs` to a yaw-only `PlayerFacing` with shortest-arc exponential smoothing, previous-yaw retention on invalid aim, and death suppression.
- [x] 2.2 Add unit tests for smoothing convergence, frame-rate independence, wrap across `+/-pi`, invalid aim retention, and death.
- [x] 2.3 Remove `MoveZone`, idle catch-up, twist tracking, and their tests from `facing.rs`.

## 3. Anchor Locomotion

- [x] 3.1 Replace fixed forward/back/left/right weights in `crates/presentation/src/anim/director.rs` with aim-space angle projection, anchor bracketing, single-anchor and no-anchor fallbacks, and a fixed-size weight buffer.
- [x] 3.2 Implement the shared normalized phase clock, dominant-anchor selection, per-clip seek on dominant change, and speed scaling (including sprint multiplier).
- [x] 3.3 Cache anchor clip nodes and durations at bind; drop anchors with missing clips with one warning and no per-frame allocation.
- [x] 3.4 Add unit tests for exact anchor poses, cardinal diagonals at 50/50, eight-anchor exact diagonals, seam continuity, missing-anchor fallback, dominant selection, and phase seek math.

## 4. Camera-Relative Input

- [x] 4.1 Derive the movement basis from the camera yaw in `crates/presentation/src/input/system.rs`, falling back to world axes when no camera exists.
- [x] 4.2 Add unit tests for camera-relative and fallback mappings.

## 5. Weapon Pose & Dual-Hand IK

- [x] 5.1 Add a weapon module (e.g. `crates/presentation/src/anim/weapon.rs`) that spawns the mounted weapon under the visual-root anchor, resolves `Grip`/`Foregrip`/`Magazine`/`Muzzle` once, and caches entity IDs.
- [x] 5.2 Compose the anchor transform from base offset, relaxed/aim pose, sway, recoil, reload, and swap offsets; update the weapon transform in `PostUpdate` before IK.
- [x] 5.3 Generalize `crates/presentation/src/anim/ik.rs` to solve either arm chain to a socket transform, with reach clamping, pole handling, and hand alignment.
- [x] 5.4 Implement optional shoulder/clavicle settle with a catalog weight and clamp, then solve right arm to `Grip` and left arm to `Foregrip`.
- [x] 5.5 Drive recoil from `WeaponFiredMessage`, reload pose from `WeaponInventory::reload_progress()`, and swap lowering/raise plus socket re-resolution from `WeaponSwappedMessage`.
- [x] 5.6 Disable weapon pose, socket targeting, and IK while dead or when no weapon is mounted; restore locomotion arm motion when unarmed.
- [x] 5.7 Extend `crates/presentation/src/anim/rig.rs` to cache both arm chains and shoulder bones, and remove the rig `WeaponSocket`/`Foregrip` assumptions.
- [x] 5.8 Add unit tests for pose composition, recoil decay/clamp, reload progression at 0/0.5/1 and cancellation, swap timing, two-arm independence, reach clamping, and shoulder settle bounds.

## 6. Graph, Schedule & Lifecycle

- [x] 6.1 Simplify `crates/presentation/src/anim/graph.rs` to base locomotion, optional upper-body additive action, and death override; remove aim/reload clip nodes.
- [x] 6.2 Update `crates/presentation/src/anim/bind.rs` to bind anchors, both IK chains, and weapon sockets, and to rebuild/reset phase and kick state on hot reload.
- [x] 6.3 Register the new systems in `crates/presentation/src/anim/mod.rs` in the order: drive → `AnimationSystems` → weapon pose → arm IK → root yaw, all before transform propagation and only in `Playing`/`GameOver`.
- [x] 6.4 Delete `crates/presentation/src/anim/twist.rs`, its schedule registration, and all cone/twist remnants; verify graph handles are dropped exactly once.
- [x] 6.5 Add unit tests for graph layer construction, optional-clip handling, death suppression, and system-set membership.

## 7. Assets *(gated: preview approval before export)*

- [x] 7.1 Author missing anchor clips on the existing 31-bone rig: backward walk and left/right strafe, in place and looping; aim and reload clips are not required because those poses are procedural.
- [x] 7.2 Provide separate weapon glTF assets (convert the supplied `game/assets/models/weapons/guns/*.fbx` meshes or author new ones, subject to approval) with `Grip`, `Foregrip`, `Magazine`, and `Muzzle` sockets, `-Z` forward, `+Y` up, meters, grip at origin.
- [x] 7.3 Render previews of the new clips and weapons and obtain user approval before replacing `game/assets/models/player.glb` or adding weapon models.
- [x] 7.4 Export approved assets, verify bone count, clip names, socket names, triangle budgets, and in-place root motion with `tools/blender/inspect_glb.py`.
- [x] 7.5 Wire the final clip and model paths in `animation.ron` and `weapons.ron`.

## 8. Documentation, Verification & Supersession

- [x] 8.1 Update `docs/01-combat-movement-and-weapons.md` with always-aim facing, camera-relative movement, and procedural weapon handling.
- [x] 8.2 Update `docs/07-technical-architecture-save-and-roadmap.md` with the anchor locomotion model, layer stack, and `PostUpdate` order; remove cone/sprint-facing text.
- [x] 8.3 Update `docs/09-player-asset-baseline.md` with the final clip set, separate weapons, and socket contract; add a weapon rig/animation contract document if the detail does not fit.
- [x] 8.4 Run targeted presentation tests and formatting/lints; do not run workspace-wide cargo checks unless needed.
- [x] 8.5 Archive `openspec/changes/player-upper-lower-rotation` with a superseded note and remove its obsolete unfinished tasks (done 2026-11-12 as `archive/2026-11-12-player-upper-lower-rotation`).
