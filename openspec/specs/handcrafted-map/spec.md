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

### Requirement: Procedural ground elevation offset

The simulation SHALL spawn procedural arena ground geometry below $Y = 0.0$ (specifically with top surface at $Y = -0.05\,\text{m}$) so that authored map ground surfaces at $Y = 0.0$ render without z-fighting or depth flickering.

#### Scenario: Procedural ground top surface sits below zero elevation
- **WHEN** `spawn_ground` executes during arena initialization
- **THEN** the ground entity transform is positioned at $Y = -0.55\,\text{m}$, placing its upper cuboid face at $Y = -0.05\,\text{m}$

### Requirement: Camera line-of-sight wall occlusion fading

The presentation system SHALL perform raycasting from the active `Camera3d` towards the survivor character center. Any static world obstacle intersecting the ray between the camera and the survivor MUST have its visual material switched to a semi-transparent material ($\text{alpha} = 0.3$, `AlphaMode::Blend`) and tagged with `OccludedWall`. When an obstacle ceases to intersect the camera line of sight, its original material MUST be restored and `OccludedWall` removed.

#### Scenario: Wall occluding survivor becomes semi-transparent
- **WHEN** an obstacle mesh intercepts the ray between the camera and survivor
- **THEN** the obstacle entity receives `OccludedWall` holding its original material handle, and its active material is replaced with a blend material with alpha 0.3

#### Scenario: Wall no longer occluding survivor is restored
- **WHEN** an occluded obstacle entity is no longer intersected by the camera ray
- **THEN** its original material handle is restored and `OccludedWall` is removed
