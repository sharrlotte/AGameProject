## MODIFIED Requirements

### Requirement: Map spatial markers for gameplay entities

The simulation SHALL query named nodes in the map hierarchy to configure gameplay entities:
- `Spawn_Player`: sets the starting survivor horizontal position and orientation while preserving the survivor's configured aim/base height (`cfg.aim_height`).
- `Spawn_Enemy` / `Spawn_Enemy_{Kind}`: creates spawner anchor entities for horde escalation.
- `Extract_Zone`: positions the extraction zone sensor and visual pad.

#### Scenario: Survivor spawns at authored marker
- **WHEN** the arena scene finishes loading with a `Spawn_Player` marker
- **THEN** the survivor entity is positioned at that marker's X and Z coordinates with Y set to the configured player elevation (`cfg.aim_height`)

#### Scenario: Enemy spawners anchor escalation
- **WHEN** enemy spawn systems execute
- **THEN** spawn candidates are derived from active `Spawn_Enemy` markers

### Requirement: Building interior roof cutaway occlusion

The simulation SHALL detect when the survivor enters an interior trigger volume prefixed with `Trigger_Interior_{id}`. Trigger entities SHALL receive a Rapier sensor `Collider` and `ActiveEvents::COLLISION_EVENTS` with visual mesh hidden. While the survivor is inside, any mesh prefixed with `Roof_{id}` or `UpperWall_{id}` MUST have its visibility hidden. When the survivor exits, the roof meshes MUST return to their default visibility.

#### Scenario: Trigger entity receives collider and collision events
- **WHEN** a map scene node named `Trigger_Interior_{id}` is processed
- **THEN** it receives `Sensor`, a valid Rapier `Collider`, `ActiveEvents::COLLISION_EVENTS`, and `Visibility::Hidden`

#### Scenario: Survivor enters building and roof hides
- **WHEN** the survivor enters the sensor volume of `Trigger_Interior_GasStation`
- **THEN** all meshes linked to `GasStation` (e.g. `Roof_GasStation`) are hidden

#### Scenario: Survivor exits building and roof restores
- **WHEN** the survivor exits `Trigger_Interior_GasStation`
- **THEN** all linked roof meshes return to visible

## ADDED Requirements

### Requirement: Camera raycast line-of-sight occlusion fading

The presentation camera system SHALL cast a ray from the active camera toward the survivor's focus point. Any obstacle or exterior wall entity intercepting the line of sight before the survivor MUST have its visibility set to hidden or dithered/transparent while occluding the survivor.

#### Scenario: Exterior wall occludes camera view of survivor
- **WHEN** an obstacle entity intercepts the camera ray to the survivor
- **THEN** the occluding entity is marked occluding and its visibility is hidden or faded

#### Scenario: Survivor moves out from behind exterior wall
- **WHEN** the camera ray to the survivor is unobstructed
- **THEN** previously occluded obstacles are restored to visible
