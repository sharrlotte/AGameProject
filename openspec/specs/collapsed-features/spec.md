# Collapsed Features

## Requirements

### Requirement: Every feature is a component on the block entity

After spawning, a block entity SHALL carry all of its features as components;
no system creates additional child entities on the normal spawn path.

#### Scenario: Thruster block exists after spawn
- **WHEN** a `ThrusterSmall` block spawns via `BlockData::spawn`
- **THEN** the entity has `Block`, `Thruster`, `PowerUsageComputed`, and no
  child entities beyond the turret-barrel exception

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

The editor SHALL spawn the same component layout (features on the block,
barrel child for turrets only).

#### Scenario: Editor-created block placed
- **WHEN** a user places a block in the editor
- **THEN** the placed entity has no child entities except a barrel for
  turret blocks
