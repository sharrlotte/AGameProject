# Simulation Sets

## Purpose

Systems register into one documented set pipeline so execution ordering is explicit and reviewable.

## Requirements
### Requirement: Simulation has a named, reviewable ordering contract

All systems participating in the fixed and frame pipelines SHALL be registered into an explicit set with a documented position in the pipeline, and all plugins SHALL rely on those sets instead of ad-hoc schedule placement where ordering matters.

#### Scenario: Fresh setup ordering
- **WHEN** the application starts
- **THEN** `PreUpdate/Input`, `FixedPreUpdate/Stats`, `FixedUpdate/Ai`, `FixedUpdate/Control`, `FixedUpdate/Weapons`, the Rapier step, `FixedPostUpdate/Damage`, `FixedPostUpdate/Death`, `PostUpdate/Present`, and `Last/Cleanup` run in that order

#### Scenario: New plugin registers ordering
- **WHEN** a plugin registers a system that depends on another plugin's output
- **THEN** it declares the dependency through the shared sets, not by placing the system in a schedule and hoping for the best

### Requirement: No shared enum tags with multiple meanings

The codebase SHALL NOT contain a single enum used both as a spawn/lifetime
marker and as a semantic key in unrelated systems.

#### Scenario: Selection and outcome queries use typed components
- **WHEN** the game scene iterates ships for selection counts or outcome checks
- **THEN** it queries component markers, not a shared tag enum

#### Scenario: EntityType removed
- **WHEN** a block is spawned by a feature spawn path
- **THEN** no system tags it with a shared "Block" enum, and selection and
  battle outcome code query dedicated marker components instead

### Requirement: Shared mutable state is typed, not grab-bag

Resources that more than one unrelated system mutate SHALL be split into
purpose-named resources with a single owner per field.

#### Scenario: Camera and battle systems stop sharing `GameVars`
- **WHEN** the camera lerps toward the followed ship and the battle queries
  the deploy payload
- **THEN** each uses its own typed resource (`CameraTarget`, `DeployPayload`)
  and does not depend on the other's data

### Requirement: Docs match the new foundation

`docs/architecture.md` SHALL describe the named simulation contract, the typed resources, the marker-component taxonomy, `DespawnOnExit` state scoping, and the damage to death to ship-mutation chain, replacing the old scheduling tables.

#### Scenario: Architecture doc read after migration
- **WHEN** a contributor opens `docs/architecture.md` after this change
- **THEN** the scheduling table lists the new sets and their members, the state scopes, and no longer references removed symbols such as `GameVars`, `SceneEntity`, or `Simulate`
