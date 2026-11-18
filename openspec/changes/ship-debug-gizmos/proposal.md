## Why

Debug ship text and AI steering gizmos currently render for every ship, so a busy battle becomes unreadable and an enemy's behavior cannot be inspected in isolation. The only selection mechanism is a transient, player-only Ctrl+drag box, leaving no persistent "ship under inspection" for a renderer to key off. A selection-scoped, dev-feature-gated overlay makes ship diagnostics useful without changing gameplay or shipping debug code.

## What Changes

- Add a dev-feature-only ship debug module: click any ship (player or enemy) to toggle it in a multi-selection; Escape clears the selection.
- Show the per-ship `Text2d` debug dump only for selected ships, spawning it once and updating on change instead of rebuilding every fixed tick.
- Add a dedicated gizmo renderer that draws only for selected ships: AI behavior geometry, hull/bounds with a core marker, heading/velocity/force, and each turret's ray to `aim_point`.
- Move AI behavior gizmo drawing out of `update_ship_behavior` into the renderer; put the shared geometry in an always-compiled helper so production steering and the dev renderer use one source of truth.
- **BREAKING**: Debug text moves from the `debug_assertions` gate to `feature = "dev"`. Plain debug builds without `--features dev` no longer show it; `cargo dev` is unaffected.
- Keep Ctrl+drag box selection player-only and unchanged; its `EntitySelectEvent` stays independent of the new dev selection.
- Drop despawned ships from the selection so no stale entity references remain.

## Capabilities

### New Capabilities
- `ship-debug`: Dev-feature-only click selection of any ship, persistent multi-selection state, selection-scoped debug text, and four families of ship gizmos with selection-lifecycle cleanup.

### Modified Capabilities
- `performance`: The debug-text toggle requirement changes from a `DebugOverlay` toggle to `feature = "dev"` plus selected ships, with no per-tick rebuild for unselected ships.

## Impact

- New modules: `content/ship/debug/{mod.rs, component.rs, system.rs, lib.rs}` and an always-compiled shared geometry helper in `content/ship`.
- Modified modules: `content/ship/{component.rs, system.rs, mod.rs}`, `content/mod.rs`, and the `/docs` specs for UI, controls, scenes, simulation, and architecture.
- Registration: dev-gated `ShipDebugPlugin`; text in `FixedUpdate/Present`, gizmos in `PostUpdate/Present`, click observer on ship roots, all scoped to `SceneState::Game`.
- Dependencies: none added. Uses existing `SpritePickingPlugin` (via `SpritePlugin` in `DefaultPlugins`), `PointerClick` bubbling, and `Gizmos`.
