## Why

Currently the player character rotates rigidly as a single unit toward the aim direction (or movement velocity while sprinting), and `player.glb` lacks authored sideways and backward walk clips (`walk_back` reuses forward `Man_Walk`, while `strafe_l` and `strafe_r` are missing). Splitting upper-body and lower-body rotation lets the upper body track the mouse cursor at all times while the lower body walks forward naturally within human spine twist limits and transitions to realistic strafing or backpedaling when the angle becomes unhuman.

## What Changes

- **Independent Upper/Lower Body Rotation**: Decouple upper-body yaw (always facing the mouse aim direction) from lower-body yaw (`PlayerVisualRoot`), distributing the residual yaw twist across the spine (`Hips`, `Abdomen`, `Torso`).
- **Natural Twist Threshold & Idle Catch-Up**: Keep the lower body planted while idle until upper-body twist exceeds a configurable human comfort limit (`±45°`), then smoothly rotate the lower body to catch up with hysteresis.
- **Cone-Based Locomotion Orientation**: While moving, keep the lower body aligned to movement (`walk_fwd` / `Man_Run`) within the forward twist cone (`≤ 45°`), align to aim (`strafe_l` / `strafe_r`) in the sideways cone (`45°–135°`), and face opposite movement (`walk_back`) with comfortable residual spine twist in the backward cone (`> 135°`).
- **Always-Aim Upper Body During Sprint**: Update facing rules so the upper body still faces the mouse cursor while sprinting; forward-cone sprinting uses `Man_Run` while sideways/backward-cone sprinting plays `strafe_l`/`strafe_r`/`walk_back` scaled up to sprint speed.
- **Spine Twist Before IK**: Apply procedural spine twist in `PostUpdate` after `AnimationSystems` and before `PlayerIkSet` so left support-hand IK automatically follows the rotated rifle foregrip.
- **Data-Driven Twist Config**: Add hot-reloadable spine twist parameters (max twist angle, hysteresis, blend duration, bone weights) to `AnimCatalog` and `game/assets/config/player/animation.ron`.
- **Authored Directional Walk Clips**: Author `HumanArmature|Man_WalkBack`, `HumanArmature|Man_StrafeL`, and `HumanArmature|Man_StrafeR` in `game/assets/models/player.glb` via Blender (with preview renders approved before export) and wire `walk_back` to `HumanArmature|Man_WalkBack` in `animation.ron`.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `player-animation`: Replace whole-body facing with split upper/lower rotation, idle twist threshold + catch-up turn, cone-based lower-body orientation (normal walk, sideways strafe, backward walk), always-mouse upper-body facing during sprint with speed-scaled directional fallback, procedural spine twist before IK, and hot-reloadable twist config.
- `player-rig`: Extend the player glTF animation clip contract to require a dedicated in-place backward walk clip (`walk_back` / `HumanArmature|Man_WalkBack`) alongside `strafe_l` and `strafe_r`.

## Impact

- **Code (`crates/presentation`)**: `src/anim/facing.rs`, `src/anim/director.rs`, `src/anim/catalog.rs`, `src/anim/rig.rs`, `src/anim/mod.rs`, plus a new `src/anim/twist.rs` module.
- **Config (`game/assets/config/player`)**: `animation.ron` (new twist section, `walk_back` clip name).
- **Assets (`game/assets/models`)**: `player.glb` (new `Man_WalkBack`, `Man_StrafeL`, `Man_StrafeR` clips on existing 31-bone rig).
- **Documentation (`docs`)**: `docs/01-combat-movement-and-weapons.md` (sprint/aim facing rules) and `docs/09-player-asset-baseline.md` (new clip baseline).
