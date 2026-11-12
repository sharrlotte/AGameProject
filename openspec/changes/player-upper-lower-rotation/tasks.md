## 1. Animation Catalog & Twist Configuration

- [x] 1.1 Add `TwistConfig` (`enabled`, `max_twist_deg`, `hysteresis_deg`, `idle_turn_stop_deg`, `twist_blend_secs`, `bones` with `(name, weight)` entries) to `AnimCatalog` in `crates/presentation/src/anim/catalog.rs`, with validation and defaults.
- [x] 1.2 Add the `twist` section to `game/assets/config/player/animation.ron` (`max_twist_deg: 45.0`, `hysteresis_deg: 5.0`, `idle_turn_stop_deg: 5.0`, `twist_blend_secs: 0.10`, `bones: [("Hips", 0.25), ("Abdomen", 0.35), ("Torso", 0.40)]`) and update `catalog.rs` unit tests.

## 2. Dual Facing & Cone Locomotion

- [x] 2.1 Refactor `PlayerFacing` and `sync_player_facing` in `crates/presentation/src/anim/facing.rs` to track `lower` rotation, `upper` aim rotation, smoothed `twist_rad`, `MoveZone` (`Forward | Sideways | Backward`), and `idle_turning` state with hysteresis and death reset.
- [x] 2.2 Add unit tests in `crates/presentation/src/anim/facing.rs` covering idle comfort threshold + catch-up turn, forward/sideways/backward moving cones, cone boundary hysteresis, invalid aim retention, and death twist zeroing.
- [x] 2.3 Update `drive_player_animation` in `crates/presentation/src/anim/director.rs` to project velocity onto `PlayerFacing.lower` (not single facing) and play speed-scaled directional clips when sprinting outside the forward cone; add unit tests for sprint cone fallback.

## 3. Procedural Spine Twist & Schedule Wiring

- [x] 3.1 Extend `crates/presentation/src/anim/rig.rs` and `bind.rs` to resolve and cache the configured spine twist bone entities on `PlayerRig` during initial bind and `AnimCatalog` hot reload.
- [x] 3.2 Create `crates/presentation/src/anim/twist.rs` implementing `apply_upper_body_twist` (world-space vertical yaw distribution converted back to local bone rotations via `rig::compose_world`), plus unit tests verifying twist distribution and zeroing when dead.
- [x] 3.3 Register `apply_upper_body_twist` in `crates/presentation/src/anim/mod.rs` (`PostUpdate`, after `AnimationSystems` and before `PlayerIkSet`) and verify `cargo test -p presentation` passes.

## 4. Blender Directional Walk Clips & GLB Export

- [x] 4.1 Author in-place looping `HumanArmature|Man_WalkBack`, `HumanArmature|Man_StrafeL`, and `HumanArmature|Man_StrafeR` clips on the existing 31-bone `HumanArmature` in Blender (`player.glb`).
- [x] 4.2 Render preview frames/animations of the three new clips and present them for user approval before overwriting `game/assets/models/player.glb`.
- [ ] 4.3 Export the approved `game/assets/models/player.glb`, verify bone count and clips with `tools/blender/inspect_glb.py`, and update `walk_back` in `game/assets/config/player/animation.ron` to `"HumanArmature|Man_WalkBack"`.

## 5. Documentation & Verification

- [ ] 5.1 Update `docs/01-combat-movement-and-weapons.md` (independent upper-body mouse aiming including sprint, lower-body cone rules) and `docs/09-player-asset-baseline.md` (new `Man_WalkBack`, `Man_StrafeL`, `Man_StrafeR` clips and twist bone notes).
- [ ] 5.2 Run targeted unit tests (`cargo test -p presentation`) after final edits and verify all tests pass without warnings.
