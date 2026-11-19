## Why

Currently, clicking a ship to inspect its stats only exists in `dev` builds via `ShipDebugPlugin` as a raw, rotating world-space `Text2d` dump. Players in production builds have no way to click a ship and see its hull layout, block health, or live combat/power/mobility stats. Promoting ship selection and inspection into a proper HUD feature makes ship status readable and actionable in all builds while keeping enemy inspection and debug gizmos gated to `dev`.

## What Changes

- **New `InspectedShip` selection model (all builds)**:
  - Introduce a single-ship `InspectedShip(Option<Entity>)` resource in `SceneState::Game`, decoupled from `Ctrl+drag` fleet box-selection (`EntitySelectEvent`).
  - Left-clicking a player ship in-world or clicking a ship in the left `ShipList` sets `InspectedShip`; pressing `Escape` or despawning the ship clears it.
  - Clicking an enemy ship (`!team.is_player()`) is gated to `#[cfg(feature = "dev")]` only.
- **New Ship Inspector HUD panel (all builds)**:
  - Spawn a dedicated bottom-right HUD panel in `SceneState::Game` (visible only when `InspectedShip` is `Some`), leaving `ShipList` and `SelectablePanel` unobstructed.
  - Render a block-image hull diagram using `BlockRegistry` sprite handles (`Feature::Texture` / `Feature::Turret`) with placement rotation/flip, rebuilt only when `InspectedShip` changes or `ShipChanged` fires.
  - Update block HP color tints and live ship stats (Hull HP, Power production/demand/stored, Cargo, Mobility, Combat range/ammo/radar, AI Role/Doctrine/Tactic) in-place on a short interval (~50–100ms) for near-real-time feedback without layout churn.
- **Refocus `ShipDebugPlugin` (`#[cfg(feature = "dev")]`)**:
  - **BREAKING** (dev-only): Remove `ShipDebugSelection`, `ShipDebugSelected`, `ShipDebugSelectionCleared`, and world-space `ShipUIRoot` `Text2d`.
  - `draw_ship_debug_gizmos` reads `InspectedShip` directly.
  - Dev builds append debug-only fields (`Steering`, `Skill`, raw `Ammo`/`AmmoState` details) to the Ship Inspector HUD.

## Capabilities

### New Capabilities
- `ship-inspector`: Production single-ship click selection (`InspectedShip`), bottom-right HUD inspector panel with block sprite diagram, and near-real-time ship stats (with dev-only enemy selection and extra debug fields).

### Modified Capabilities
- `ship-debug`: Replace `ShipDebugSelection` and world-space `ShipUIRoot` `Text2d` with `InspectedShip` and HUD dev-section integration; keep `draw_ship_debug_gizmos` dev-only and driven by `InspectedShip`.

## Impact

- `core/src/content/ship/debug/` (refactored to gizmos + dev-only helpers, or `InspectedShip` moved to `core/src/content/ship/` / `core/src/scene/game/`)
- `core/src/content/ship/component.rs` (remove `ShipUIRoot` spawn; observe `on_ship_click` in all builds)
- `core/src/scene/game/` (split/add `inspector` submodule for the bottom-right HUD panel, block `ImageNode` builder, and ~50–100ms live stat/tint updater; update `on_ship_button` in `ShipList`)
- `docs/architecture.md`, `docs/scenes.md`, `docs/simulation.md`, `docs/ui.md` (updated to reflect the new spec)
