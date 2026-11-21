## MODIFIED Requirements

### Requirement: Turret barrel is the only child-entity exception

Feature-owned child entities SHALL be limited to `Visual` (block transform/mirror wrapper), extra `Texture` sprite layers, turret barrels (`TurretSprite` under `Visual`), shield barriers (`Barrier` under block), and overdrive fields (`Field` under block). All feature child entities (barrels, barriers, overdrive fields) SHALL be spawned reactively via `On<Add, T>` observers rather than imperative feature-insert code, and SHALL carry presentation/sensor data only with logic owned by components on the block entity.

#### Scenario: Turret block spawns barrel under Visual via observer
- **WHEN** a turret block spawns and commands flush
- **THEN** the barrel `TurretSprite` is spawned under `Visual` by the `On<Add, TurretBarrelSpec>` observer, `TurretBarrel(barrel)` is recorded on the block, and no ownership or targeting logic lives on the barrel child

#### Scenario: Shield and booster blocks spawn their child entities via observers
- **WHEN** a shield block or booster block spawns and commands flush
- **THEN** `Shield` spawns one `Barrier` child via `On<Add, Shield>` and `Booster` spawns one `Field` child via `On<Add, Booster>`, with parent links recorded on the block entity

### Requirement: Editor spawn path matches the collapsed layout

Placed blocks in the Ship Editor SHALL reuse the same `Visual`, `TurretBarrelSpec`, and `Booster` observer-driven visual child spawning and shared material helpers rather than duplicating imperative barrel and overdrive field spawning code.

#### Scenario: Editor-placed turret and booster spawn matching visuals
- **WHEN** a user places a turret or booster block in the Ship Editor
- **THEN** its barrel sprite or overdrive field visual is created through the shared observer/helper path with accurate per-radius overdrive ring proportions
