## MODIFIED Requirements

### Requirement: Modules SHALL NOT form import cycles

No two source modules in `core/src` SHALL import from each other. Dependencies between a parent module and its feature submodules SHALL point in one direction only. Leaf types consumed by feature submodules (`Health`, `ShootPattern`, `TargetingStrategy`, `ShootingProjectile`, `TurretMirror`, `BlockStats`) SHALL live in leaf modules rather than in `content::block::component`.

#### Scenario: Block and feature modules are one-way
- **WHEN** the module dependency graph of `content::block` and its `feature` submodules is inspected
- **THEN** no `content::block::feature::*` submodule imports types from `content::block::component`

#### Scenario: Feature enum does not reach back to its owner
- **WHEN** `content::block::feature::component` is compiled
- **THEN** it has no `use` dependency on `content::block::component`

#### Scenario: Turret and shield submodules have no reverse block imports
- **WHEN** `feature/turret/component.rs`, `feature/turret/system.rs`, and `feature/shield/system.rs` are inspected
- **THEN** none of them import from `content::block::component`

### Requirement: Feature insertion SHALL receive consumed values only

Feature insertion SHALL take a `SpawnCtx` carrying only the consumed layout/runtime values (`base`, `visual`, `pixel_size`, `team`, `config`) and never `&BlockDef`, so a feature cannot depend on unrelated definition fields or unused positional arguments.

#### Scenario: Pixel sizing is passed as a value
- **WHEN** a block's features are inserted into an entity
- **THEN** `Feature::install` passes a `SpawnCtx` containing the `Vec2` pixel size computed once by the caller

#### Scenario: No BlockDef in feature implementations
- **WHEN** `feature/sprite/component.rs`, `feature/turret/component.rs`, or `feature/shield/component.rs` is inspected
- **THEN** none imports `BlockDef`, none accepts `&BlockDef` in an `install` signature, and no feature `install` method carries unused positional arguments
