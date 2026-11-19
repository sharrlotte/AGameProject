## Context

When a ship breaks apart in `spawn_debris_from_component` (`core/src/content/body/mutation.rs`) or is restored as debris in `DebrisSnapshot::restore` (`core/src/save/battle/lib.rs`), detached blocks are reparented to a `Debris` entity and their `BlockOwner.body` is updated to that `Debris` entity. However, blocks keep their active feature components (`Turret`, `TurretState`, `Radar`, `Booster`, shooting patterns, `Demand`, etc.), and active block systems do not check whether the owning body is `Debris` or `Ship`. As a result, turrets on debris continue scanning, aiming, firing, charging, and keeping active laser/pulling beams visible, while detached boosters can remain `powered = true`.

## Goals / Non-Goals

**Goals:**
- Ensure blocks owned by a `Debris` body are automatically marked `Disabled` across all detachment, recursive debris split, and save-restore paths.
- Reset active block state and hide in-flight laser/pulling beam visuals immediately when a block is disabled.
- Exclude `Disabled` blocks from all active block systems (turrets, shooting patterns, radars, boosters, thrusters) with zero per-tick iteration overhead for debris.
- Keep `Disabled` reusable for future mechanics (e.g., EMP, overheating, manual toggling).

**Non-Goals:**
- Stripping or removing functional components (`Turret`, `Radar`, `Booster`) from debris blocks.
- Adding a user-facing UI toggle or EMP mechanic in this change.
- Changing how projectiles already in flight behave after a turret detaches.

## Decisions

### 1. `Disabled` marker synced via `On<Insert, BlockOwner>` observer
- **Decision**: Define a `Disabled` marker component in `core/src/content/block/` and register an `On<Insert, BlockOwner>` observer that inserts `Disabled` whenever `owner.body` has `Debris`.
- **Rationale**: Combines `Debris` as the single source of truth (no risk of forgetting a manual `.insert(Disabled)` in a future debris path) with archetype-level `Without<Disabled>` filtering in systems (zero per-tick cost for debris, no changes needed to standalone unit tests that omit `BlockOwner`/`Ship`).
- **Alternatives considered**:
  - *Querying `ships.contains(owner.body)` in every system*: Adds `&BlockOwner` + `Query<(), With<Ship>>` boilerplate to ~10 systems, visits debris blocks every tick, and breaks existing unit tests that spawn turrets without `BlockOwner`/`Ship`.
  - *Manual `.insert(Disabled)` in `spawn_debris_from_component` and `DebrisSnapshot::restore`*: Risks future drift if another path attaches blocks to `Debris`.

### 2. State & visual cleanup via `On<Add, Disabled>` observer
- **Decision**: Register an `On<Add, Disabled>` observer that resets `TurretState` (`active = false`, `can_fire = false`, `target = None`), clears `Radar.targets`, sets `BoosterState.powered = false` and `Demand.value = 0.0`, resets `LaserBeamPatternState.shooting` to `Cooldown` / hides its `beam` entity (`Visibility::Hidden`), and hides `PullingBeamPatternState.beam` (`Visibility::Hidden`).
- **Rationale**: Prevents mid-shot laser or pulling beams from remaining frozen `Visibility::Visible` on drifting debris and ensures boosters do not retain stale `powered = true` state.

### 3. Gate all active block systems with `Without<Disabled>`
- **Decision**: Add `Without<Disabled>` to queries in:
  - `radar_scan` (`core/src/content/block/radar/system.rs`)
  - `update_turret_demand`, `advance_turret_charge`, `update_turret_target`, `turret_tracking` (`core/src/content/block/feature/turret/system.rs`)
  - `handle_single_shot`, `handle_burst_shot`, `handle_laser_beam`, `handle_pulling_beam` (`core/src/content/bullet/system.rs`)
  - `power_boosters`, `apply_boosts` (`core/src/content/block/feature/power/booster.rs`)
  - `update_thruster_demand` (`core/src/content/block/feature/thruster/system.rs`)
- **Rationale**: Uniformly disables all active block behaviors on debris while leaving `Health` and damage/collision intact.

## Risks / Trade-offs

- **[Observer ordering on `BlockOwner` insert]** → `On<Insert, BlockOwner>` checks `debris.contains(owner.body)`. Both `spawn_debris_from_component` and `DebrisSnapshot::restore` spawn the `Debris` entity via `spawn_debris_body` before calling `set_block_owner` / `spawn_block`, so `Debris` is already present when the observer fires. We will add a unit/integration test covering both detachment and save-restore to verify this invariant.
- **[`Disabled` not removed if `BlockOwner` later changes to `Ship`]** → Currently blocks never re-attach from `Debris` to `Ship`, and `Disabled` may later be used by non-debris mechanics (e.g. EMP). Leaving removal out avoids prematurely coupling `Disabled` exclusively to `Debris`.
