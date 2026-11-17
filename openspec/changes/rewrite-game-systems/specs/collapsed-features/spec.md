## MODIFIED Requirements

### Requirement: Every feature is a component on the block entity

After spawning, a block entity SHALL carry all of its features as components; no system creates additional child entities on the normal spawn path.

#### Scenario: Thruster block exists after spawn
- **WHEN** a `ThrusterSmall` block spawns from a `BlockPlacement`
- **THEN** the entity has `BlockRef`, `BlockOwner`, `Thruster`, its applicable power-efficiency runtime state, and no child entities beyond the turret-barrel exception

### Requirement: Editor spawn path matches the collapsed layout

The editor SHALL spawn the same component layout (features on the block, barrel child for turrets only).

#### Scenario: Editor-created block placed
- **WHEN** a user places a block in the editor from a `BlockPlacement`
- **THEN** the placed entity has `BlockRef` and `BlockOwner` and no child entities except a barrel for turret blocks
