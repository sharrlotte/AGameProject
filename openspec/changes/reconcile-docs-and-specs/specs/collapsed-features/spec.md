## MODIFIED Requirements

### Requirement: Every feature is a component on the block entity

After spawning, gameplay logic for every block feature SHALL live on components attached to the block entity; child entities SHALL be restricted strictly to presentation and sensor overlays (`Visual`, extra `Texture` sprites, turret barrel `TurretSprite`, shield `Barrier`, overdrive `Field`, and transient hit/beam VFX).

#### Scenario: Thruster block carries stat components on the block entity
- **WHEN** a `thruster_small` block spawns from a `BlockPlacement`
- **THEN** the block entity carries `BlockRef`, `BlockOwner`, `ThrustPowerBase`, `ThrustPowerComputed`, `RotateSpeedBase`, `RotateSpeedComputed`, `Consumer`, `Demand`, `Supply`, and `Boost`, with only its `Visual` child for sprite presentation
