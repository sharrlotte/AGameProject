# Module Boundaries

## Purpose

Module dependencies are acyclic and one-way, and a component receives only the layout data it consumes rather than the definition that owns it.

## Requirements

### Requirement: Modules SHALL NOT form import cycles

No two source modules in `core/src` SHALL import from each other. Dependencies between a parent module and its feature submodules SHALL point in one direction only.

#### Scenario: Block and feature modules are one-way
- **WHEN** the module dependency graph of `content::block` and its `feature` submodules is inspected
- **THEN** `content::block::feature` submodules do not import types defined in `content::block::component`

#### Scenario: Feature enum does not reach back to its owner
- **WHEN** `content::block::feature::component` is compiled
- **THEN** it has no `use` dependency on `content::block::component`

### Requirement: Feature insertion SHALL receive consumed values only

Feature insertion SHALL take the specific layout values the feature needs, not the owning `BlockDef`, so a feature cannot depend on unrelated definition fields.

#### Scenario: Pixel sizing is passed as a value
- **WHEN** a block's features are inserted into an entity
- **THEN** each `insert` method receives a `Vec2` pixel size computed once by the caller

#### Scenario: No BlockDef in feature implementations
- **WHEN** `feature/sprite/component.rs` or `feature/turret/component.rs` is inspected
- **THEN** neither imports `BlockDef` and neither accepts `&BlockDef` in an `insert` signature
