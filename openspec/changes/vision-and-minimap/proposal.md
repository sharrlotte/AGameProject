## Why

With `smart-utility-ai` in place, ships already scan within `Capability.max_radar_range` and publish shared sightings, decaying `last_known` positions, and incoming-fire `alerts` into `TacticalBoard` / `TeamIntel`. However, the player currently still has omniscient visibility of every enemy ship across the map and no tactical radar minimap to read radar range or intel states.

## What Changes

- Enforce player fog-of-war visibility driven by the player team's shared `TeamIntel.contacts`: undetected enemy ships are hidden (`Visibility::Hidden`), excluded from offscreen edge indicators (`draw_indicators`), and cannot be clicked/selected or inspected in dev ship debug while undetected.
- Keep projectiles (bullets, missiles, lasers) visible even when fired by an undetected enemy so incoming fire remains fair and readable alongside `TeamIntel.alerts`.
- Add a top-right HUD minimap widget (`GameScenePlugin` / dedicated minimap plugin) that stays north-up (no map rotation), renders at a hot-reloadable fixed world scale from `.ron`, centers on the player ship using the fallback anchor chain (`ControlledShip` → `CameraTarget` → first live player-team ship → camera position), draws a rotating player heading arrow at center, draws the centered ship's `Capability.max_radar_range` ring, and plots allied dots, live detected enemy dots (`contacts`), fading ghost dots (`last_known`), and incoming-fire alert pings (`alerts`).
- Add hot-reloadable minimap config (size, world radius, margins, update/pool bounds) in `.ron` with validation.
- Update `/docs` (`ui.md`, `ai.md`, `config.md`) to specify player vision and minimap behavior.

## Capabilities

### New Capabilities
- `player-vision`: Fog-of-war visibility for enemy ships driven by shared player `TeamIntel`, gating world rendering, offscreen indicators, and selection while keeping projectiles visible.
- `hud-minimap`: Always-on top-right north-up tactical minimap displaying the rotating player arrow, center ship's radar range ring, allied dots, live detected enemy dots, fading last-known ghosts, and incoming-fire alert pings at a configurable fixed world scale.

### Modified Capabilities

## Impact

- **Systems & Modules**:
  - New `core/src/content/ship/vision/` plugin/module for syncing enemy `Visibility` from `TacticalBoard` (`SimulationSet::Present`).
  - `core/src/content/ship/indicator/mod.rs`: filter enemy offscreen indicators through player `TeamIntel.contacts`.
  - `core/src/content/selectable/` and `core/src/content/ship/debug/`: ignore hidden/undetected enemy ships.
  - New `core/src/scene/game/minimap/` plugin/module for the top-right HUD minimap UI and pooled dot/ring/arrow updates.
  - `core/src/config/` & `core/assets/config/base/`: hot-reloadable minimap configuration and validation.
- **Dependencies**: Consumes existing `TacticalBoard`, `TeamIntel`, and `Capability.max_radar_range` from `core/src/content/ship/ai/`. No new crates required.
- **Documentation**: Updates `docs/ui.md`, `docs/ai.md`, and `docs/config.md`.
