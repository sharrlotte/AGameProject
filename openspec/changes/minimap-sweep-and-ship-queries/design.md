## Context

Two clusters of issues were surfaced during exploration:
1. **Entity-query leaks**: `apply_selection()` and `on_ship_click()` in `core/src/content/ship/inspect/system.rs` query `(&Team, &Visibility)` without `With<Ship>`, and `apply_selection()` falls through when `ships.get(event.entity)` returns `Err`. Likewise, `sync_enemy_visibility()` in `core/src/content/ship/vision/system.rs` queries `(Entity, &Team, &mut Visibility)` without `With<Ship>`, which can hide enemy non-ship entities (e.g., bullets, debris, blocks).
2. **Minimap UX & per-frame UI churn**: `update_minimap()` in `core/src/scene/game/minimap/system.rs` currently:
   - Anchors to a ship instead of the player camera, rendering a second inner `MinimapRing` circle and a `MinimapArrow` (`"\u{25b2}"`) whose missing glyph in Bevy's default font renders as a green rectangle.
   - Plots `contacts`, `alerts`, and `last_known` simultaneously, producing duplicate red + orange dots for the same enemy.
   - Allocates `Vec`/`HashMap`s and mutates all 128 `MinimapMarker` UI `Node`s every frame.

## Goals / Non-Goals

**Goals:**
- Ensure ship inspection (`apply_selection`, `on_ship_click`) and enemy visibility (`sync_enemy_visibility`) only operate on `Ship` entities.
- Make the minimap always centered on the active `Camera2d` position, with a single outer border circle and no center rectangle/arrow.
- Scale minimap world radius dynamically to `active_radar_range * radar_scale_factor` (`1.2` default) with a clean ship-priority fallback.
- Eliminate duplicate enemy dots by plotting allies, live `TeamIntel.contacts`, and short-lived lost-contact flashes only.
- Update dot positions on a 1-second sweep cadence, add subtle `BoxShadow` dot glow, and flash lost contacts N times before disappearing.
- Eliminate per-frame UI mutations on idle minimap frames and reuse `Local` scratch buffers in minimap and AI systems.

**Non-Goals:**
- Changing AI internal `TeamIntel.alerts` or `TeamIntel.last_known` behavior (only minimap rendering of them changes).
- Adding custom WGSL shaders for minimap rendering (Bevy UI `BoxShadow` + `BackgroundColor` suffice).

## Decisions

### 1. Strict `With<Ship>` filtering and early-continue in `apply_selection()`
- **Decision**: Add `With<Ship>` to `apply_selection()`, `on_ship_click()`, and `sync_enemy_visibility()`. In `apply_selection()`, change `if let Ok(...)` to `let Ok((team, visibility)) = ships.get(event.entity) else { continue; };`.
- **Rationale**: Prevents non-ship entities (blocks, projectiles, debris) from ever being inspected or hidden by ship-vision logic.

### 2. Camera-centered minimap & `radar_range * 1.2` scaling
- **Decision**: Always center the minimap on `Camera2d` world translation. Remove `MinimapRing` and `MinimapArrow` entities. Resolve `active_radar_range` via `ControlledShip.controlling` → `CameraTarget.following` → max `Capability.max_radar_range` among alive player ships, multiplied by `config.radar_scale_factor` (default `1.2`), falling back to `config.fallback_world_radius` when no player ship has radar.
- **Rationale**: Eliminates the second circle and the green tofu glyph while keeping the minimap scale automatically matched to the active ship's radar reach.

### 3. Single dot per enemy + lost-contact flash state
- **Decision**: Plot only allied ships, live `TeamIntel.contacts`, and minimap-local `LostContact` flash entries (dropping `alerts` and `last_known` from minimap). On each 1-second sweep, any previously tracked enemy entity absent from `contacts` (and still alive or recently lost) enters `lost_flashes` with its last known position and team color, blinking `lost_flash_count` times over `lost_flash_secs` before removal.
- **Rationale**: Guarantees 1 enemy = 1 dot and gives clear visual feedback when an enemy leaves radar range.

### 4. Two-stage minimap pipeline (1s sweep + change-gated presentation)
- **Decision**:
  - `sweep_minimap` runs on a configurable `refresh_secs` timer (default `1.0s`), reusing `Local` scratch buffers to build `MinimapSnapshot` (marker positions, colors, and active `lost_flashes`).
  - `render_minimap` runs in `PostUpdate` and exits early unless `MinimapSnapshot` changed, `MinimapConfigValue` changed, or `lost_flashes` is non-empty. Active markers get a subtle `BoxShadow` glow; lost-contact markers modulate alpha/visibility in a square/triangle wave for `lost_flash_count` blinks.
- **Rationale**: Gives the intended 1-second radar sweep feel and cuts idle-frame minimap UI layout cost to zero.

### 5. Reusable `Local` scratch buffers in AI & booster systems
- **Decision**: Convert per-tick `HashMap::new()` / `Vec::new()` in `sensor.rs`, `board.rs`, `readiness.rs`, `scorer.rs`, and `booster.rs` to `Local<...>` buffers cleared via `.clear()` at the start of each run.
- **Rationale**: Retains allocated capacity across frames, eliminating repeated heap allocations.

## Risks / Trade-offs

- **Camera panning far from player ships**: Because the minimap is always camera-centered, panning far away in FreeLook can move contacts outside `radar_range * 1.2` and clip them at the border. → *Mitigation*: Expected behavior for a camera-centered local radar; `fallback_world_radius` ensures valid bounds when no player ships remain.
- **`BoxShadow` cost on many dots**: `BoxShadow` on 128 hidden markers could add UI render work if left active. → *Mitigation*: Only apply/enable `BoxShadow` on visible markers (or set `BoxShadow::default()` when hidden) and cap `max_markers` sensibly in `minimap.ron`.
