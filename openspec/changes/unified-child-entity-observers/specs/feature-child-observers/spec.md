## ADDED Requirements

### Requirement: Feature insertion inserts data components only
Block feature installation SHALL insert pure data components onto the block entity and SHALL NOT spawn child entities imperatively inside feature `install` methods.

#### Scenario: Turret installation inserts barrel spec without spawning barrel directly
- **WHEN** `TurretSpec::install` runs on a block entity
- **THEN** it inserts `Turret`, `TurretState`, `Radar`, pattern/power/ammo components, and `TurretBarrelSpec`, and spawns no barrel entity directly inside `install`

### Requirement: All feature child entities spawn synchronously via Bevy On<Add, T> observers
Turret barrels (`On<Add, TurretBarrelSpec>`), shield barriers (`On<Add, Shield>`), and overdrive fields (`On<Add, Booster>`) SHALL be spawned by Bevy 0.20 `On<Add, T>` observers registered in their owning feature plugins. Each observer SHALL guard against duplicate child creation via an explicit parent-to-child link component (`TurretBarrel`, `Shield.barrier`, `OverdriveField`) and SHALL execute synchronously during command flush.

#### Scenario: Chunk-restored ship gets shield barrier and turret barrel on the same flush
- **WHEN** a ship containing a turret and a shield is restored during `stream_chunks` and `world.flush()` completes
- **THEN** both `TurretBarrel` and `Shield.barrier` exist and are parented before the next fixed physics tick

#### Scenario: Re-inserting Booster does not duplicate overdrive field
- **WHEN** a `Booster` component is replaced on a block that already has `OverdriveField`
- **THEN** no second `Field` child entity is spawned

#### Scenario: Headless tests without render asset resources do not panic
- **WHEN** `Shield` or `Booster` is spawned in a minimal test `App` without `Assets<Mesh>` or `FieldAssets`
- **THEN** the observer returns cleanly without panicking

### Requirement: Disabling a shield block immediately deactivates its barrier via observer
An `On<Add, Disabled>` observer in `ShieldPlugin` SHALL immediately disable the barrier's `CollisionGroups` (`Group::NONE`) and hide its `Visibility` (`Visibility::Hidden`), matching the observer pattern used by turrets, boosters, radar, and beams, and making debris restoration order-independent.

#### Scenario: Debris shield block never exposes an active barrier
- **WHEN** a block with `Shield` receives `BlockOwner` pointing to a `Debris` entity (triggering `Disabled`) in the same command flush
- **THEN** either no barrier is spawned or its `CollisionGroups` are `Group::NONE` and `Visibility` is `Visibility::Hidden` immediately after flush
