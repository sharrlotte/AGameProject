## Context

`smart-utility-ai` introduced `Capability.max_radar_range`, `Sensor`, and `TacticalBoard { teams: HashMap<u8, TeamPlan> }`, where each team maintains a shared `TeamIntel { contacts, last_known, alerts }`. Currently, all enemy ships remain `Visibility::Visible` at all times, `draw_indicators` renders offscreen edge arrows for every ship regardless of detection, and the Game HUD has no radar minimap.

Note that `GameScenePlugin`'s `update_ship_list` already filters by `team.is_player()`, and its `top_bar` is gated under `#[cfg(debug_assertions)]`, so the minimap must live in its own always-on top-right HUD container.

## Goals / Non-Goals

**Goals:**
- Make player-team `TeamIntel` (`TacticalBoard.teams[DEFAULT_TEAM_ID].intel`) the single source of truth for player vision and enemy stealth.
- Hide undetected enemy ships (`Visibility::Hidden`) and prevent UI/debug leaks (offscreen `draw_indicators`, `Selectable`, dev `ShipUIRoot` click inspection) while keeping fired projectiles visible.
- Add an always-on top-right HUD minimap (`MinimapPlugin` under `core/src/scene/game/minimap/`) with north-up orientation, fixed hot-reloadable world scale, rotating center player arrow, center ship radar range ring, allied dots, live detected enemy dots, fading `last_known` ghosts, and incoming-fire `alerts`.
- Follow `AGENTS.md` module conventions (`mod.rs`, `component.rs`, `system.rs`, `config.rs`) and hot-reloadable `.ron` config validation.

**Non-Goals:**
- Changing AI radar scanning or `TeamIntel` mechanics (already handled by `smart-utility-ai`).
- Fog-of-war terrain/exploration map or chunk streaming (handled by `world-chunk-streaming`).
- Drawing radar range rings for non-centered allied ships (clutter reduction; only the centered ship's ring is drawn).
- Rotating the minimap with ship heading (strictly north-up).

## Decisions

### 1. Single Source of Truth for Player Vision (`TacticalBoard`)
- **Choice**: A dedicated `VisionPlugin` (`core/src/content/ship/vision/`) runs `sync_enemy_visibility` in `PostUpdate` (`SimulationSet::Present`) while in `SceneState::Game`. It reads `TacticalBoard.teams.get(&DEFAULT_TEAM_ID)` and sets `Visibility::Visible` on enemy ships present in `intel.contacts`, and `Visibility::Hidden` on all other enemy ships.
- **Rationale**: Reuses the exact radar detection already computed by `update_sensors_and_intel`, automatically supports fleet-shared vision across multiple allied ships, and toggles child block sprites automatically via Bevy's `InheritedVisibility`.
- **Alternatives considered**: Reading only `ControlledShip`'s `Sensor.local_contacts` (breaks in `FreeLook` and ignores allied scouts); hiding only on minimap (contradicts radar-gated enemy stealth).

### 2. Anti-Leak Filtering (Indicators, Selection, Dev Debug)
- **Choice**:
  - `draw_indicators` (`core/src/content/ship/indicator/mod.rs`) reads `TacticalBoard` and skips non-player ships not present in player `intel.contacts`.
  - `Selectable` / selection systems and dev `on_ship_click` (`core/src/content/ship/debug/`) ignore entities whose `Visibility` is `Visibility::Hidden` (or non-player ships absent from player `intel.contacts`).
  - Projectiles (`Bullet`, `Missile`, `Laser`) remain `Visibility::Visible` unconditionally so blind/long-range enemy fire is readable and pairs naturally with `TeamIntel.alerts`.
- **Rationale**: Prevents undetected enemies from leaking position via edge arrows or click-probing while keeping combat fair.

### 3. Top-Right Minimap Anchor & Rendering (`core/src/scene/game/minimap/`)
- **Choice**:
  - Anchored top-right via `PositionType::Absolute` (`top`, `right` from config) inside the `Game` scene root (`DespawnOnExit(SceneState::Game)`), independent of the `#[cfg(debug_assertions)]` `top_bar`.
  - **Fallback anchor chain**: `ControlledShip.controlling` → `CameraTarget.following` → first live player-team ship (sorted deterministically by `Ship.id`) → active `Camera2d` world position (`FreeCamera` mode: hides player arrow and radar ring, still plots allied/intel dots relative to camera).
  - **Pure Bevy UI rendering with pooled markers**: Circular/rounded minimap container with `Overflow::clip()`, a centered radar ring `Node` (`BorderRadius::MAX`, `BorderColor`) sized to `(radar_range / world_radius) * minimap_radius_px * 2.0`, a rotating center player arrow node (`UiTransform::from_rotation`), and a bounded pool of reusable dot `Node`s updated in-place each frame (zero per-frame entity spawn/despawn churn).
  - **Coordinate mapping (North-Up)**: `offset = target_pos - anchor_pos`; `norm = offset / world_radius`; clip if `norm.length_squared() > 1.0`; map `+Y` world to `-Y` UI (`Vec2::new(norm.x, -norm.y) * minimap_radius_px`).
  - **Intel styling**:
    - Allies: player team color dots.
    - Live `contacts`: bright enemy team color dots.
    - `last_known`: faded enemy-colored ghost dots whose alpha scales linearly with `remaining_ttl / max_memory_secs`.
    - `alerts`: amber/red incoming-fire ping dots whose alpha scales with `remaining_ttl / max_alert_ttl_secs`.
- **Rationale**: Pooled Bevy UI nodes are fast, crisp, resolution-independent, require no custom shaders or secondary cameras, and avoid allocation/despawn churn.
- **Alternatives considered**: Auto-scaling minimap to `1.25× radar_range` (hides radar upgrades/losses); custom render-to-texture minimap camera (overkill for dots + ring + arrow).

### 4. Hot-Reloadable Minimap Config (`core/assets/config/base/minimap.ron`)
- **Choice**: Load `MinimapConfig` (`size_px`, `margin_px`, `world_radius`, `dot_size_px`, `arrow_size_px`, `max_markers`) via `RonAssetLoader` with validation (all sizes/radii positive, `max_markers >= 1`) and `AssetEvent` bridge mirroring `AiConfig` / `GameConfig`.
- **Rationale**: Zero hardcoded layout or scale constants (`AGENTS.md`), hot-reloadable for quick tuning.

### 5. Module Layout (`AGENTS.md` Compliant)
```
core/src/content/ship/vision/
├── mod.rs          # VisionPlugin (PostUpdate, SimulationSet::Present)
└── system.rs       # sync_enemy_visibility + unit tests

core/src/scene/game/minimap/
├── mod.rs          # MinimapPlugin (OnEnter(SceneState::Game), PostUpdate SimulationSet::Present)
├── component.rs    # MinimapRoot, MinimapRadarRing, MinimapPlayerArrow, MinimapMarkerPool, MinimapAnchor
├── config.rs       # MinimapConfig / MinimapConfigValue asset loader + validation
└── system.rs       # setup_minimap, update_minimap, resolve_minimap_anchor, world_to_minimap + unit tests
```

## Risks / Trade-offs

- **Physics vs. Visibility**: Undetected enemies remain active Rapier rigid bodies, so players can collide with or hit them blindly. → Expected and desirable for physical consistency; a hit or incoming shot reveals combat feedback naturally.
- **Marker Pool Exhaustion**: Large battles with many allies, contacts, ghosts, and alerts could exceed `max_markers`. → Prioritize rendering order: live `contacts` → allies → `alerts` → `last_known`, and make `max_markers` configurable in `minimap.ron` (default e.g. `128`).
- **Frame Ordering**: `sync_enemy_visibility` and `update_minimap` must run after `SimulationSet::Ai` updates `TacticalBoard`. → Running in `PostUpdate` (`SimulationSet::Present`) guarantees `FixedUpdate` (`SimulationSet::Ai`) has already published current `TeamIntel`.
