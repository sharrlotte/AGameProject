# hud-minimap Specification

## Purpose

Top-right north-up circular minimap in `SceneState::Game` centered on `Camera2d`, dynamically scaled by `active_radar_range * radar_scale_factor`, with real-time glowing contacts and out-of-range lost-contact flashes.

## Requirements

### Requirement: Camera-centered north-up minimap with single border
The `Game` scene SHALL display an always-on, north-up circular minimap in the top-right corner (`DespawnOnExit(SceneState::Game)`) centered on the active `Camera2d` world position. The minimap SHALL render only a single circular outer border (`MinimapRoot`) and SHALL NOT render an inner radar ring (`MinimapRing`) or center arrow (`MinimapArrow`).

#### Scenario: Minimap centers on camera with single border and no center rectangle
- **WHEN** `SceneState` enters `Game`
- **THEN** the minimap centers on `Camera2d` world coordinates, displays only the single outer border circle, and has no inner ring or center arrow/rectangle

### Requirement: Dynamic radar-scaled world radius
The minimap SHALL compute its displayed world radius as `active_radar_range * radar_scale_factor` (default `1.2`), resolving `active_radar_range` via `ControlledShip.controlling` → `CameraTarget.following` → maximum `Capability.max_radar_range` among alive player-team ships, and falling back to `fallback_world_radius` from `minimap.ron` when no alive player ship has positive radar range.

#### Scenario: Scale uses controlled ship radar range times 1.2
- **WHEN** `ControlledShip.controlling` is `Some(ship)` with `max_radar_range = 2500.0` and `radar_scale_factor = 1.2`
- **THEN** the minimap maps offsets against a world radius of `3000.0`

#### Scenario: FreeLook falls back to followed or max player ship radar
- **WHEN** `ControlledShip.controlling` is `None` and `CameraTarget.following` is `None` while alive player ships have radar ranges `1500.0` and `2500.0`
- **THEN** the minimap maps offsets against `2500.0 * radar_scale_factor`

### Requirement: Real-time glowing contacts, radar-green border, and lost-contact flash
The minimap SHALL render a radar-green outer border without a rotating hand, inner ring, or center arrow, and SHALL update the camera center, allied player-team ship dots, and live `TeamIntel.contacts` in real time every frame (omitting `TeamIntel.alerts` and `TeamIntel.last_known`). Visible marker dots SHALL render with a subtle `BoxShadow` glow, and when an enemy contact leaves `TeamIntel.contacts`, its dot SHALL blink `lost_flash_count` times over `lost_flash_secs` at its last world-anchored position before disappearing.

#### Scenario: Enemy appears as a single glowing dot updated in real time
- **WHEN** 2 enemy ships are in `TeamIntel.contacts`
- **THEN** the minimap displays 2 glowing enemy dots (no duplicate orange alert dots) updated in real time inside the radar-green border

#### Scenario: Enemy leaving radar range flashes before disappearing
- **WHEN** a tracked enemy contact leaves `TeamIntel.contacts`
- **THEN** its marker blinks `lost_flash_count` times over `lost_flash_secs` at its last world-anchored position before becoming hidden

### Requirement: Minimap configuration is data-driven and validated
`MinimapConfig` SHALL be loaded from `core/assets/config/base/minimap.ron` with hot-reload support and validated for positive `size_px`, `fallback_world_radius`, `radar_scale_factor`, `refresh_secs`, `dot_size_px`, `lost_flash_secs`, `glow_blur_px`, non-negative `margin_px`, `lost_flash_count >= 1`, and `max_markers >= 1`.

#### Scenario: Invalid minimap config is rejected
- **WHEN** `MinimapConfig` has `refresh_secs <= 0.0`, `radar_scale_factor <= 0.0`, or `lost_flash_count == 0`
- **THEN** validation returns an explicit error
