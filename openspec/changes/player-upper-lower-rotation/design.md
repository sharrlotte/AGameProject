## Context

Currently, `crates/presentation/src/anim/facing.rs` rotates the entire `PlayerVisualRoot` toward `AimDirection` (or horizontal velocity when sprinting) as a single rigid object. Meanwhile, `crates/presentation/src/anim/director.rs` computes locomotion weights by projecting velocity onto that single `facing` vector, and `game/assets/models/player.glb` only contains `Man_Walk` (forward), `Man_Run`, `Man_Idle`, and `Man_Death` — `walk_back` reuses `Man_Walk`, and `strafe_l`/`strafe_r` do not exist yet.

Inspecting `game/assets/models/player.glb` also establishes the live skeleton hierarchy:
```text
Bone
└── Body              (animated by Man_Idle / Man_Walk / Man_Run)
    ├── Hips          (parents Abdomen -> Torso -> Neck -> Head + Arms; NOT legs)
    │   └── Abdomen
    │       └── Torso
    ├── UpperLeg.L -> LowerLeg.L
    └── UpperLeg.R -> LowerLeg.R
```
Because `Body` is the seam above both legs and `Hips`, rotating `PlayerVisualRoot` controls the lower body (`Body` + legs), while applying yaw twist across `Hips`, `Abdomen`, and `Torso` rotates the upper body without disturbing leg orientation.

## Goals / Non-Goals

**Goals:**
- Keep the upper body continuously facing the mouse cursor (`AimDirection`), including while sprinting.
- Keep the lower body planted during small idle aim adjustments (`≤ ±45°`), smoothly catching up with hysteresis once exceeded.
- While moving, animate normal forward walking (`walk_fwd` / `Man_Run`) within the natural `±45°` torso-twist cone, switch to pure sideways strafing (`strafe_l` / `strafe_r`) in the `45°–135°` cone, and switch to backward walking (`walk_back`) with comfortable `≤ ±45°` spine twist when moving away from the mouse (`> 135°`).
- Apply procedural spine twist after `AnimationSystems` and before `PlayerIkSet` so left support-hand IK automatically follows the rotated rifle.
- Author `HumanArmature|Man_WalkBack`, `HumanArmature|Man_StrafeL`, and `HumanArmature|Man_StrafeR` in `player.glb` with preview renders approved before export.
- Keep all thresholds, blend times, and bone weights hot-reloadable in `game/assets/config/player/animation.ron`.

**Non-Goals:**
- Authoring separate `sprint_back`, `sprint_strafe_l`, or `sprint_strafe_r` clips (sideways/backward sprint reuses `strafe_l`/`strafe_r`/`walk_back` scaled up to sprint playback speed).
- Modifying simulation movement speeds, collision shapes, or `AimDirection` calculation.
- Changing enemy facing or animation behavior.

## Decisions

### D1. Dual-facing state in `PlayerFacing` (`anim/facing.rs`)
Split `PlayerFacing` into:
- `lower: Quat` — lower-body world-relative visual yaw written to `PlayerVisualRoot.rotation`.
- `upper: Quat` — upper-body target yaw tracking `AimDirection` (retaining previous yaw when aim is invalid).
- `twist_rad: f32` — signed yaw delta (`upper - lower` wrapped to `[-π, π]`), smoothed with exponential damping and zeroed when dead.
- `idle_turning: bool` and `zone: MoveZone` (`Forward | Sideways | Backward`) — hysteresis state so boundary angles do not flicker frame-to-frame.

Why: Keeping facing math in one resource/component (`PlayerFacing`) lets `drive_player_animation` read `PlayerFacing::lower` directly to compute character-local locomotion weights instead of re-deriving facing independently.

Alternative considered: Computing twist inside `director.rs` and keeping `facing.rs` single-yaw. Rejected because locomotion blend weights depend on lower-body facing, so splitting the source of truth risks divergence.

### D2. Cone-based lower-body orientation with hysteresis
Let $\Delta = \text{wrap\_pi}(\theta_{\text{aim}} - \theta_{\text{move}})$, with configurable `max_twist_deg` (default `45°`) and `hysteresis_deg` (default `5°`):
1. **Moving (`speed > move_speed_threshold`)**:
   - **Forward cone** (`|Δ| ≤ max_twist`): lower target = $\theta_{\text{move}}$, twist = $\Delta$. Legs run `walk_fwd` (or `Man_Run` if sprinting).
   - **Sideways cone** (`max_twist < |Δ| ≤ 180° - max_twist`): lower target = $\theta_{\text{aim}}$, twist settles to `0°`. `Weights::from_local` produces `strafe_l` / `strafe_r`.
   - **Backward cone** (`|Δ| > 180° - max_twist`): lower target = $\theta_{\text{move}} + \pi$ (opposite of velocity), residual twist = $\text{wrap\_pi}(\theta_{\text{aim}} - (\theta_{\text{move}} + \pi))$ which stays within `±max_twist`. `Weights::from_local` produces `walk_back`.
   - Switching zones requires crossing the threshold by `hysteresis_deg` to eliminate jitter along 45°/135° diagonals.
2. **Idle (`speed ≤ move_speed_threshold`)**:
   - Lower body remains stationary while `|upper - lower| ≤ max_twist`.
   - Once exceeded, `idle_turning` engages and `lower` smoothly slerps toward `upper` until `|upper - lower| ≤ idle_turn_stop_deg` (default `5°`).

Alternative considered: Clamping lower body to `aim ± max_twist` in the sideways cone. Rejected per user choice so sideways strafing plays pure `strafe_l`/`strafe_r` with zero spine strain.

### D3. Sprint interaction with always-mouse upper body
Because the upper body always tracks `AimDirection` even when sprinting:
- `director.rs` uses `Man_Run` (`sprint`) when `stamina.is_sprinting && moving && zone == Forward`.
- When `stamina.is_sprinting && moving && zone != Forward`, `director.rs` runs directional locomotion (`strafe_l`/`strafe_r`/`walk_back`) with playback speed scaled to `catalog.sprint_playback_speed` (respecting `speed_scale_max` or sprint scale).

Why: Avoids breaking the spine or abandoning mouse aim during tactical sprints while keeping the clip budget to 3 new directional walk clips.

### D4. World-space spine twist before IK (`anim/twist.rs`)
Schedule order in `PostUpdate`:
```text
AnimationSystems
  ─▶ apply_upper_body_twist  (after AnimationSystems, before PlayerIkSet)
  ─▶ PlayerIkSet             (apply_support_hand_ik)
  ─▶ sync_player_facing      (before TransformSystems::Propagate)
```
For each weighted spine bone (`Hips: 0.25`, `Abdomen: 0.35`, `Torso: 0.40` by default in `animation.ron`, resolved into `PlayerRig` at bind/hot-reload):
1. Compose bone and parent transforms in `PlayerVisualRoot` local space via `rig::compose_world`.
2. Apply `Quat::from_rotation_y(twist_rad * weight)` around `PlayerVisualRoot`'s vertical `Vec3::Y` axis.
3. Convert back into the bone's local rotation (`parent_world.rotation.inverse() * twisted_world`).

Why: Animated clips introduce pitch and roll on `Abdomen`/`Torso`; rotating around local `Vec3::Y` directly would lean the character sideways when twisted. Rotating in `PlayerVisualRoot` space guarantees pure horizontal yaw regardless of clip lean, and running before `PlayerIkSet` lets `apply_support_hand_ik` keep the left hand pinned to `Foregrip`.

Alternative considered: Additive animation pose clips for left/right twist. Rejected because procedural yaw handles arbitrary continuous angles (`-45°..+45°`), works uniformly over idle/walk/run/aim/reload, and avoids extra Blender pose blending.

### D5. Data-driven `TwistConfig` in `AnimCatalog`
Add a `twist: TwistConfig` struct to `AnimCatalog` (`crates/presentation/src/anim/catalog.rs`) and `game/assets/config/player/animation.ron` with `#[serde(default)]` support and validation (`max_twist_deg` in `0.0..=90.0`, non-empty bone list when enabled, `twist_blend_secs >= 0.0`). Cache resolved spine bone entities on `PlayerRig` during `bind_player_animation_players` and `sync_animation_catalog_hot_reload`.

### D6. Authoring `Man_WalkBack`, `Man_StrafeL`, `Man_StrafeR` in `player.glb`
Use Blender (`mcp-for-blender` / Python script on the existing 31-bone `HumanArmature`) to author three in-place looping locomotion clips:
- `HumanArmature|Man_WalkBack`: grounded backward stride with reverse heel-toe roll and subtle torso counterbalance.
- `HumanArmature|Man_StrafeL` / `HumanArmature|Man_StrafeR`: clean lateral step cycles without knee/foot clipping.
Render multi-frame preview images for user approval before overwriting `game/assets/models/player.glb`, then update `walk_back` in `animation.ron` from `HumanArmature|Man_Walk` to `HumanArmature|Man_WalkBack`.

## Risks / Trade-offs

- **Rotating in place during idle turn can look slightly planted ("sliding feet")** → Mitigation: Smooth `facing_blend_secs` damping keeps catch-up turns crisp (~0.12s) rather than slow sliding; future turn-in-place clips remain additive without changing `PlayerFacing`.
- **Rapid 180° mouse flicks across Forward/Backward boundaries** → Mitigation: `hysteresis_deg` prevents cone chatter, and `facing_blend_secs` slerps `lower` cleanly instead of snapping.
- **Additive twist on `Hips` vs `Abdomen`/`Torso` could slightly shift leg root if `Hips` moved legs** → Mitigation: Verified in `player.glb` node tree that `UpperLeg.L`/`UpperLeg.R` are siblings of `Hips` under `Body`, not children of `Hips`; rotating `Hips` affects only the spine upward.
- **Blender clip quality / foot skate** → Mitigation: Require rendered preview approval before exporting `player.glb`, and validate with `tools/blender/inspect_glb.py`.

## Migration Plan

1. Add `TwistConfig` to `AnimCatalog` and `animation.ron` (with `walk_back` still valid via fallback/missing-clip warning until GLB export).
2. Implement `PlayerFacing` dual-yaw + cone/idle hysteresis in `anim/facing.rs` and update `director.rs` to read `PlayerFacing` lower rotation + sprint directional fallback.
3. Implement `anim/twist.rs` + `rig.rs` spine bone lookup, schedule between `AnimationSystems` and `PlayerIkSet`, and cover with unit tests.
4. Author `Man_WalkBack`, `Man_StrafeL`, `Man_StrafeR` in Blender, render previews for user approval, export `player.glb`, and update `animation.ron` `walk_back` role.
5. Update `docs/01-combat-movement-and-weapons.md` and `docs/09-player-asset-baseline.md`.
