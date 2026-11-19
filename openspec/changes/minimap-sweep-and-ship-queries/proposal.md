## Why

`apply_selection()` and related ship-visibility queries lack a `With<Ship>` filter, allowing non-ship entities (such as blocks, bullets, or debris) to be matched or selected. At the same time, the minimap currently renders two concentric circles, a green missing-glyph rectangle at its center, duplicate red/orange dots for the same enemy, and mutates all 128 marker UI nodes every frame instead of running on a 1-second radar sweep.

## What Changes

- Constrain `apply_selection()`, `on_ship_click()`, and `sync_enemy_visibility()` queries with `With<Ship>` and skip non-ship entities in `apply_selection()`.
- **BREAKING** Change minimap center to always track the active `Camera2d` world position rather than anchoring to a ship, removing the center heading arrow (`MinimapArrow`) and the second inner radar ring (`MinimapRing`) so only the single outer border circle renders.
- Scale minimap world radius dynamically to `active_radar_range * radar_scale_factor` (`1.2` by default), resolving `active_radar_range` via `ControlledShip` → `CameraTarget.following` → max `Capability.max_radar_range` among alive player ships → config fallback.
- Plot only allied ships, live `TeamIntel.contacts`, and short-lived lost-contact flashes on the minimap (dropping `TeamIntel.alerts` and `TeamIntel.last_known` from minimap rendering so each enemy produces at most one dot).
- Decouple minimap updates into a 1-second radar sweep (`refresh_secs`) and a change-gated UI/flash sync, adding subtle `BoxShadow` dot glow and multi-blink lost-contact flashing before a dot disappears.
- Replace per-frame `HashMap`/`Vec` allocations in the minimap and AI sensor/board/readiness/scorer/booster systems with reusable `Local` buffers.

## Capabilities

### New Capabilities
- `hud-minimap`: Camera-centered north-up minimap with single border, `radar * 1.2` dynamic scaling, 1-second sweep updates, glowing dots, and out-of-range lost-contact flashes.

### Modified Capabilities
- `ship-inspector`: Restrict selection application and click handling strictly to `Ship` entities.
- `performance`: Eliminate per-frame UI node mutations on idle minimap frames and reuse `Local` scratch buffers in minimap and AI tick systems.

## Impact

- `core/src/content/ship/inspect/system.rs` (`apply_selection`, `on_ship_click`)
- `core/src/content/ship/vision/system.rs` (`sync_enemy_visibility`)
- `core/src/scene/game/minimap/` (`config.rs`, `component.rs`, `system.rs`, `mod.rs`) and `core/assets/config/base/minimap.ron`
- `core/src/content/ship/ai/` (`sensor.rs`, `board.rs`, `readiness.rs`, `scorer.rs`) and `core/src/content/block/feature/power/booster.rs`
- `/docs` spec updates matching the code changes
