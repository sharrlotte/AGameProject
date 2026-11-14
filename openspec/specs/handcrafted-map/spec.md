# Handcrafted Map Specification

## Purpose

Defines how the simulation loads, inspects, and binds authored glTF arena maps, including collision extraction, gameplay spatial anchors, and building interior cutaways.

## Requirements

### Requirement: Authored glTF scene map loading

The simulation SHALL load hand-crafted district maps from glTF/GLB assets when entering `Scene::Arena`. The active map path SHALL be configurable or defaulted to `maps/district_01.glb`.

#### Scenario: Map asset mounts
- **WHEN** the game enters `Scene::Arena`
- **THEN** the authored district map glTF scene is instantiated under the arena hierarchy

#### Scenario: Missing map logs error
- **WHEN** a specified map asset fails to load
- **THEN** an error is logged and the simulation falls back safely

### Requirement: Low-poly collision mesh extraction

The simulation SHALL inspect the instantiated glTF scene hierarchy and convert every node whose name begins with `Collider_` into a static Rapier collider belonging to `GROUP_WORLD`, attaching the `Obstacle` component. The visual mesh of the collider MUST be hidden so that collision proxy geometry is not rendered.

#### Scenario: Collider node is registered as physics obstacle
- **WHEN** a glTF node named `Collider_Wall_01` is processed
- **THEN** it receives a static `Collider`, `Obstacle`, and participates in projectile and movement collisions

#### Scenario: Collider visual mesh is hidden
- **WHEN** a collider node is processed
- **THEN** its `Visibility` is set to `Hidden` or its rendering components are removed

### Requirement: Map spatial markers for gameplay entities

The simulation SHALL query named nodes in the map hierarchy to configure gameplay entities:
- `Spawn_Player`: sets the starting survivor position and orientation.
- `Spawn_Enemy` / `Spawn_Enemy_{Kind}`: creates spawner anchor entities for horde escalation.
- `Extract_Zone`: positions the extraction zone sensor and visual pad.

#### Scenario: Survivor spawns at authored marker
- **WHEN** the arena scene finishes loading with a `Spawn_Player` marker
- **THEN** the survivor entity is positioned at that marker's translation

#### Scenario: Enemy spawners anchor escalation
- **WHEN** enemy spawn systems execute
- **THEN** spawn candidates are derived from active `Spawn_Enemy` markers

### Requirement: Building interior roof cutaway occlusion

The simulation SHALL detect when the survivor enters an interior trigger volume prefixed with `Trigger_Interior_{id}`. While the survivor is inside, any mesh prefixed with `Roof_{id}` or `UpperWall_{id}` MUST have its visibility hidden. When the survivor exits, the roof meshes MUST return to their default visibility.

#### Scenario: Survivor enters building and roof hides
- **WHEN** the survivor enters the sensor volume of `Trigger_Interior_GasStation`
- **THEN** all meshes linked to `GasStation` (e.g. `Roof_GasStation`) are hidden

#### Scenario: Survivor exits building and roof restores
- **WHEN** the survivor exits `Trigger_Interior_GasStation`
- **THEN** all linked roof meshes return to visible
