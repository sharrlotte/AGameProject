## MODIFIED Requirements

### Requirement: Every feature is a component on the block entity

After spawning, a block entity SHALL carry all of its features as components;
no system creates additional child entities on the normal spawn path.

#### Scenario: Thruster block exists after spawn
- **WHEN** a `thruster_small` block spawns via `BlockData::spawn`
- **THEN** the entity has `Block`, `Thruster`, `Consumer`, `Demand`, `Supply`,
  `Boost`, and no child entities beyond the turret-barrel exception
