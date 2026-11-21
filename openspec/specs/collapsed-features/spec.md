# Collapsed Features

## Purpose

Block features live as components on the block entity, with the turret barrel as the only child-entity exception.
## Requirements
### Requirement: Every feature is a component on the block entity

After spawning, gameplay logic for every block feature SHALL live on components attached to the block entity; child entities SHALL be restricted strictly to presentation and sensor overlays (`Visual`, extra `Texture` sprites, turret barrel `TurretSprite`, shield `Barrier`, overdrive `Field`, and transient hit/beam VFX).

#### Scenario: Thruster block carries stat components on the block entity
- **WHEN** a `thruster_small` block spawns from a `BlockPlacement`
- **THEN** the block entity carries `BlockRef`, `BlockOwner`, `ThrustPowerBase`, `ThrustPowerComputed`, `RotateSpeedBase`, `RotateSpeedComputed`, `Consumer`, `Demand`, `Supply`, and `Boost`, with only its `Visual` child for sprite presentation

### Requirement: Turret barrel is the only child-entity exception

The turret barrel sprite (and its transform) SHALL be a single child entity
of the block; nothing else on the normal block-spawn path is a child entity.

#### Scenario: Turret block has exactly one child
- **WHEN** a turret block spawns
- **THEN** it has exactly one child (the barrel), and the barrel carries only
  `Transform` + the turret sprite; no ownership or logic is on the child

### Requirement: No transitive ownership component

`ComponentOwner` SHALL be deleted; no system queries it or inserts it.

#### Scenario: Collapsed block passes health/damage checks
- **WHEN** a block takes damage and its `Health` crosses zero
- **THEN** the block is despawned/split by existing systems and no
  `ComponentOwner` access is needed along that path

### Requirement: Editor spawn path matches the collapsed layout

The editor SHALL spawn the same component layout (features on the block, barrel child for turrets only).

#### Scenario: Editor-created block placed
- **WHEN** a user places a block in the editor from a `BlockPlacement`
- **THEN** the placed entity has `BlockRef` and `BlockOwner` and no child entities except a barrel for turret blocks

