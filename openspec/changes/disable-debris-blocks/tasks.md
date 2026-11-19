## 1. Disabled Component & Observers

- [x] 1.1 Define `Disabled` marker component in `core/src/content/block/` (exported via `core/src/content/block/mod.rs`) and register it for reflection.
- [x] 1.2 Add `On<Insert, BlockOwner>` observer in `BlockPlugin` that inserts `Disabled` onto a block when `owner.body` carries `Debris`.
- [x] 1.3 Add `On<Add, Disabled>` observer in `BlockPlugin` that resets `TurretState` (`active = false`, `can_fire = false`, `target = None`), clears `Radar.targets`, sets `BoosterState.powered = false` and `Demand.value = 0.0`, resets `LaserBeamPatternState.shooting` to `Cooldown`, and sets `LaserBeamPatternState.beam` / `PullingBeamPatternState.beam` to `Visibility::Hidden`.

## 2. Gate Active Block Systems

- [x] 2.1 Add `Without<Disabled>` to `radar_scan` in `core/src/content/block/radar/system.rs`.
- [x] 2.2 Add `Without<Disabled>` to `update_turret_demand`, `advance_turret_charge`, `update_turret_target`, and `turret_tracking` in `core/src/content/block/feature/turret/system.rs`.
- [x] 2.3 Add `Without<Disabled>` to `handle_single_shot`, `handle_burst_shot`, `handle_laser_beam`, and `handle_pulling_beam` in `core/src/content/bullet/system.rs`.
- [x] 2.4 Add `Without<Disabled>` to `power_boosters` and `apply_boosts` in `core/src/content/block/feature/power/booster.rs`.
- [x] 2.5 Add `Without<Disabled>` to `update_thruster_demand` in `core/src/content/block/feature/thruster/system.rs`.

## 3. Verification & Documentation

- [x] 3.1 Add tests verifying: (a) `On<Insert, BlockOwner>` marks blocks on `Debris` as `Disabled` (both on detach and `DebrisSnapshot::restore`), (b) `On<Add, Disabled>` resets `TurretState`/`BoosterState`/`Radar` and hides active laser/pulling beams, and (c) `Disabled` turrets/radars/boosters do not target, track, fire, or boost.
- [x] 3.2 Run relevant `cargo test` targets and verify all pass.
- [x] 3.3 Record the specification updates in `/docs` per repository rules.
