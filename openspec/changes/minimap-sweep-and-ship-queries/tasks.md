## 1. Ship Query Filters (`With<Ship>`)

- [x] 1.1 Add `With<Ship>` to `apply_selection()` and `on_ship_click()` in `core/src/content/ship/inspect/system.rs`, and change `if let Ok(...)` in `apply_selection()` to `let Ok(...) = ships.get(event.entity) else { continue; };`
- [x] 1.2 Add a unit test in `core/src/content/ship/inspect/system.rs` verifying `apply_selection()` ignores non-ship entities that have `(&Team, &Visibility)`
- [x] 1.3 Add `With<Ship>` to `sync_enemy_visibility()` in `core/src/content/ship/vision/system.rs` and add a unit test verifying non-ship entities with `Team` + `Visibility` are not hidden

## 2. Minimap Config & State

- [x] 2.1 Update `MinimapConfig` in `core/src/scene/game/minimap/config.rs` and `core/assets/config/base/minimap.ron` with `fallback_world_radius`, `radar_scale_factor` (`1.2`), `refresh_secs` (`1.0`), `lost_flash_count`, `lost_flash_secs`, and `glow_blur_px` (removing `arrow_size_px` / static `world_radius`), plus validation and config tests
- [x] 2.2 Update `core/src/scene/game/minimap/component.rs` to remove `MinimapRing`, `MinimapArrow`, `AnchorCandidate`, and `MinimapAnchor`, and add `MinimapSnapshot`, `LostContact`, and `MinimapSweepTimer`

## 3. Minimap Layout, 1s Sweep & Presentation

- [x] 3.1 Update `setup_minimap` in `core/src/scene/game/minimap/system.rs` to spawn only `MinimapRoot` (single border circle) and `MinimapMarkerLayer` (no `MinimapRing`, no `MinimapArrow` green rectangle)
- [x] 3.2 Implement `sweep_minimap` (1s timer using `Local` scratch buffers): center on `Camera2d`, resolve `active_radar_range * radar_scale_factor` via `ControlledShip` → `CameraTarget.following` → max player radar → `fallback_world_radius`, plot allies + live `TeamIntel.contacts` only, and migrate newly lost enemy contacts into `lost_flashes`
- [x] 3.3 Implement change-gated `render_minimap` in `PostUpdate`: early-return when idle, apply `BoxShadow` glow to visible markers, and animate `lost_flashes` blinking `lost_flash_count` times over `lost_flash_secs` before hiding
- [x] 3.4 Register `MinimapSnapshot` / `MinimapSweepTimer` and the `sweep_minimap` + `render_minimap` systems in `core/src/scene/game/minimap/mod.rs`, and update minimap unit tests

## 4. Reusable `Local` Buffers & Docs

- [x] 4.1 Convert per-tick `HashMap::new()` / `Vec::new()` scratch buffers to `Local<...>` with `.clear()` in `core/src/content/ship/ai/` (`sensor.rs`, `board.rs`, `readiness.rs`, `scorer.rs`) and `core/src/content/block/feature/power/booster.rs`
- [x] 4.2 Update `/docs` spec files to reflect the ship-query filters and new minimap behavior
- [x] 4.3 Run targeted unit tests (`cargo test`) and `cargo dev` verification
