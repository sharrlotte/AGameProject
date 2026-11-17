## ADDED Requirements

### Requirement: Direct entity despawn on destruction
Entities destroyed during simulation MUST be despawned directly via `Commands::despawn()` without an intermediate `Death` marker component.

#### Scenario: Ship or block is destroyed
- **WHEN** a ship, block, bullet, missile, or lifetime-expired entity is destroyed
- **THEN** the system issues `commands.entity(entity).despawn()` directly and emits `ShipDestroyed` if applicable
