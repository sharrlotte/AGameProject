## 1. Config & Assets

- [ ] 1.1 Create `core/assets/config/base/minimap.ron` with `size_px`, `margin_px`, `world_radius`, `dot_size_px`, `arrow_size_px`, and `max_markers`
- [ ] 1.2 Implement `MinimapConfig` / `MinimapConfigValue` asset loader, validation, and hot-reload bridge in `core/src/scene/game/minimap/config.rs` and wire into `ConfigPlugin`

## 2. Player Vision & Anti-Leak Filtering

- [ ] 2.1 Create `core/src/content/ship/vision/` (`mod.rs`, `system.rs`) implementing `VisionPlugin` and `sync_enemy_visibility` (`PostUpdate`, `SimulationSet::Present`) driven by `TacticalBoard.teams[DEFAULT_TEAM_ID].intel.contacts`, and register it in `ShipPlugin`
- [ ] 2.2 Filter offscreen indicators in `core/src/content/ship/indicator/mod.rs` so non-player ships absent from player `TeamIntel.contacts` are skipped
- [ ] 2.3 Guard selection (`core/src/content/selectable/`) and dev ship inspection (`core/src/content/ship/debug/`) against `Visibility::Hidden` enemy ships while keeping projectiles visible
- [ ] 2.4 Add unit tests for enemy visibility toggling (undetected, detected by ally, lost contact) and indicator/selection filtering

## 3. Top-Right HUD Minimap

- [ ] 3.1 Create `core/src/scene/game/minimap/` (`mod.rs`, `component.rs`, `system.rs`) with `MinimapPlugin` registered in `GameScenePlugin`, spawning the always-on top-right minimap root (`DespawnOnExit(SceneState::Game)`) with circular clip, centered radar range ring, rotating player arrow, and pooled marker nodes
- [ ] 3.2 Implement `resolve_minimap_anchor` (`ControlledShip` -> `CameraTarget` -> first alive player-team ship by `Ship.id` -> `Camera2d`), north-up `world_to_minimap` projection, radar ring sizing from `Capability.max_radar_range`, and marker updates for allies, live `contacts`, fading `last_known` ghosts, and fading `alerts`
- [ ] 3.3 Add unit tests for `MinimapConfig` validation, `resolve_minimap_anchor` fallback chain, north-up `world_to_minimap` clipping, and radar ring scaling

## 4. Documentation & Verification

- [ ] 4.1 Update `/docs/ui.md`, `/docs/ai.md`, and `/docs/config.md` with player vision and minimap specifications
- [ ] 4.2 Run targeted unit tests (`cargo test`) and verify in-game with `cargo dev`
