## Why

Blocks detached onto a `Debris` body currently retain their active `Turret`, `Radar`, `Booster`, and shooting pattern components, and none of those block systems check whether the owning body is `Debris`. As a result, turrets on drifting debris continue scanning, tracking, firing, and even keeping mid-shot laser/pulling beams visible, and powered boosters can remain active.

## What Changes

- Introduce a `Disabled` marker component on blocks to represent non-functional/offline block state.
- Add an `On<Insert, BlockOwner>` observer that automatically marks a block `Disabled` when its `BlockOwner.body` is a `Debris` entity (covering detachment, debris splitting, and battle save-restore).
- Add an `On<Add, Disabled>` observer that resets active block state (`TurretState`, `Radar.targets`, `BoosterState`, `Demand`) and hides any active `LaserBeamPatternState` or `PullingBeamPatternState` beam entities.
- Gate active block systems (`radar_scan`, turret targeting/tracking/power/charge, all turret shooting patterns, `power_boosters`/`apply_boosts`, and `update_thruster_demand`) with `Without<Disabled>`.

## Capabilities

### New Capabilities
- `block-disabled`: Defines the `Disabled` block marker component, automatic synchronization from `Debris` ownership, state/visual cleanup on disable, and system exclusion via `Without<Disabled>`.

### Modified Capabilities
- `debris`: Clarifies that blocks belonging to a `Debris` body are marked `Disabled` on attachment/restore and immediately cease turret, radar, booster, thruster, and beam activity.

## Impact

- `core/src/content/block/` (new `Disabled` component & observers, `radar/system.rs`, `feature/turret/system.rs`, `feature/power/booster.rs`, `feature/thruster/system.rs`)
- `core/src/content/bullet/system.rs` (`handle_single_shot`, `handle_burst_shot`, `handle_laser_beam`, `handle_pulling_beam`)
- `core/src/content/body/debris/` & `core/src/save/battle/lib.rs` (ensure `Debris` component exists on body before `BlockOwner` is inserted on blocks)
