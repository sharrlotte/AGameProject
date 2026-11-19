# Camera

## Purpose

A fixed-height orthographic view frame and borderless-fullscreen startup keep the visible world constant across window size, aspect ratio, and DPI, with zoom, pan, and the Ship Editor sharing one camera.

## Requirements
### Requirement: Fixed-height orthographic view frame

The primary `Camera2d` SHALL use an orthographic projection with `ScalingMode::FixedVertical { viewport_height: VIEW_HEIGHT }`, where `VIEW_HEIGHT` is 1080 world units. The visible world height SHALL be `VIEW_HEIGHT × ortho.scale` regardless of window size, aspect ratio, DPI scale, or fullscreen transitions. The visible world width SHALL follow the viewport aspect ratio.

#### Scenario: 16:9 viewport maps to the 1920×1080 design frame
- **WHEN** the logical viewport is 1280×720 and `ortho.scale` is 1.0
- **THEN** the camera area is 1920×1080 world units

#### Scenario: Ultrawide viewport extends width only
- **WHEN** the logical viewport is 2560×1080 and `ortho.scale` is 1.0
- **THEN** the camera area is 2560×1080 world units

#### Scenario: 4:3 viewport reduces width only
- **WHEN** the logical viewport is 1440×1080 and `ortho.scale` is 1.0
- **THEN** the camera area is 1440×1080 world units

#### Scenario: Enlarging the window does not reveal more world
- **WHEN** the window grows from 1280×720 to 1920×1080 at `ortho.scale = 1.0`
- **THEN** the camera area remains 1920×1080 world units and only pixel density changes

#### Scenario: DPI scale does not change the world frame
- **WHEN** the window moves to a monitor or OS scale with a different DPI but the same aspect ratio
- **THEN** the camera area remains `VIEW_HEIGHT × ortho.scale` world units tall

### Requirement: Borderless fullscreen startup

The primary window SHALL start in `WindowMode::BorderlessFullscreen(MonitorSelection::Primary)`.

#### Scenario: Launch covers the primary monitor
- **WHEN** the application starts
- **THEN** the primary window is borderless fullscreen on the primary monitor with no window chrome

#### Scenario: Fullscreen startup on the design display is pixel 1:1
- **WHEN** the primary monitor is 1920×1080 at 100% DPI and `ortho.scale` is 1.0
- **THEN** one world unit maps to one physical pixel

### Requirement: Zoom remains orthographic scale

Zoom SHALL continue to be expressed as `ortho.scale`, clamped to `min_zoom` and `max_zoom` loaded from `game.ron`. The visible world height SHALL be `VIEW_HEIGHT × ortho.scale`.

#### Scenario: Zooming in halves the visible height
- **WHEN** `ortho.scale` is set to 0.5
- **THEN** the visible world height is 540 world units

#### Scenario: Zooming out doubles the visible height
- **WHEN** `ortho.scale` is set to 2.0
- **THEN** the visible world height is 2160 world units

#### Scenario: Zoom clamps remain unchanged
- **WHEN** the player scrolls beyond either bound
- **THEN** `ortho.scale` is clamped to the configured `min_zoom` and `max_zoom`

### Requirement: Scale-based pan conversion

Game middle-drag SHALL translate the camera by the mouse delta multiplied by `ortho.scale`. Ship Editor left-drag SHALL use the same factor multiplied by its existing 0.5 feel multiplier. When the logical viewport height equals `VIEW_HEIGHT`, drag tracking SHALL be 1:1 with the cursor at every zoom level; other logical heights are accepted to drift.

#### Scenario: Pan is 1:1 on the design frame
- **WHEN** the logical viewport height is 1080, `ortho.scale` is 1.0, and the player middle-drags 10 points
- **THEN** the camera translation changes by 10 world units

#### Scenario: Pan scales with zoom
- **WHEN** `ortho.scale` is 2.0 and the player middle-drags 10 points
- **THEN** the camera translation changes by 20 world units

#### Scenario: Editor keeps its half-speed feel
- **WHEN** `ortho.scale` is 2.0 and the editor left-drags 10 points
- **THEN** the camera translation changes by 10 world units

### Requirement: Editor shares the camera projection

The Ship Editor SHALL use the same persistent `Camera2d` and fixed-height projection as the Game scene. No scene-specific projection switching SHALL occur, and the editor SHALL keep its existing zoom clamp `[0.1, 5.0]`.

#### Scenario: Entering the editor preserves the fixed frame
- **WHEN** `SceneState` enters `ShipEditor`
- **THEN** the visible world height remains `VIEW_HEIGHT × ortho.scale`

#### Scenario: Editor zoom clamps are preserved
- **WHEN** the editor scrolls beyond 0.1 or 5.0
- **THEN** `ortho.scale` is clamped to `[0.1, 5.0]`
