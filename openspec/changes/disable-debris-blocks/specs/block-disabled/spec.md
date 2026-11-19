## ADDED Requirements

### Requirement: Blocks can be marked Disabled
The block module SHALL provide a `Disabled` marker component. Active block systems for turrets, shooting patterns, radars, boosters, and thrusters SHALL exclude entities with `Disabled` using `Without<Disabled>`.

#### Scenario: Disabled turret does not track or fire
- **WHEN** a block with `Turret`, `Radar`, and a shooting pattern carries `Disabled` and an enemy ship is in range
- **THEN** `radar_scan`, `update_turret_target`, `turret_tracking`, and shooting systems do not target, rotate, or fire from that block

#### Scenario: Disabled booster does not boost
- **WHEN** a block with `Booster` carries `Disabled`
- **THEN** `power_boosters` and `apply_boosts` do not power it or apply its boost to nearby blocks

### Requirement: Debris ownership automatically disables blocks
Whenever `BlockOwner` is inserted or updated on a block whose `BlockOwner.body` carries `Debris`, the block SHALL automatically receive the `Disabled` component.

#### Scenario: Detached block becomes Disabled
- **WHEN** a block is detached from a ship onto a `Debris` body
- **THEN** the block receives `Disabled` automatically via `BlockOwner` synchronization

#### Scenario: Save-restored debris block is Disabled
- **WHEN** a `DebrisSnapshot` is restored with blocks owned by the restored `Debris` body
- **THEN** every restored block on that debris body carries `Disabled`

### Requirement: Disabling a block resets active state and hides beams
When `Disabled` is added to a block, its active runtime state SHALL be reset: `TurretState` SHALL have `active = false`, `can_fire = false`, and `target = None`; `Radar.targets` SHALL be cleared; `BoosterState.powered` SHALL be set to `false`; `Demand.value` SHALL be set to `0.0`; and any active `LaserBeamPatternState.beam` or `PullingBeamPatternState.beam` entity SHALL be set to `Visibility::Hidden`.

#### Scenario: Mid-shot laser beam hides on disable
- **WHEN** a laser turret is actively firing with `beam` visible and `Disabled` is added to it
- **THEN** `TurretState.active` becomes `false`, `LaserBeamPatternState.shooting` resets to cooldown, and the beam entity becomes `Visibility::Hidden`
