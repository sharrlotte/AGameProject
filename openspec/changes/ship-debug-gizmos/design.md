## Context

`ShipUIRoot` is spawned under `#[cfg(debug_assertions)]` for every ship and `update_ship_ui` rewrites `Text2d` on every fixed tick. AI steering gizmos are drawn inline in `update_ship_behavior` for every AI ship, guarded by `cfg!(debug_assertions)` inside the loop. The only game-scene selection is `SelectablePlugin`'s Ctrl+left-drag box, which filters to `team.is_player()` and emits a transient `EntitySelectEvent` consumed only by a HUD count. There is no persistent selection state for a renderer to read.

`SpritePlugin` registers `SpritePickingPlugin` through `DefaultPlugins`, and `PointerClick` auto-propagates to ancestors, so an observer on a ship root can receive clicks on its block sprites without a new picking backend. `cargo dev` is `cargo run --features dev`, and the `dev` feature already gates the inspector, remote, and asset checks.

## Goals / Non-Goals

**Goals:**

- Let a developer click any ship, including enemies, to toggle it in a persistent multi-selection.
- Restrict debug text and all ship debug gizmos to selected ships only.
- Give gizmo drawing its own system, separate from AI steering, with one shared source of geometry.
- Use a single compile-time gate, `feature = "dev"`, for all of it.
- Leave gameplay selection, camera controls, and AI behavior unchanged.

**Non-Goals:**

- Changing Ctrl+drag box selection or `EntitySelectEvent` semantics.
- Adding a picking backend, new dependency, or runtime debug-toggle UI.
- Drawing debug text or gizmos in production builds.
- Changing editor selection or turret targeting behavior.

## Decisions

1. **Gate is `feature = "dev"`, not `debug_assertions`.** All new debug code is compiled only with the dev feature, and the existing `ShipUIRoot`/`update_ship_ui`/plugin registration migrate to it. Alternative: keep `debug_assertions` (inconsistent with the rest of the dev tooling and ships debug text in every debug build). A runtime F12-style toggle was rejected as unnecessary complexity. `cargo dev` enables both gates, so the normal workflow is unchanged.

2. **Click selection uses `On<PointerClick>` on the ship root.** Sprite picking is already active, and `PointerClick` bubbles from a clicked block's visual sprite to the ship entity. The sprite picking backend requires `&Pickable` on sprite entities, so dev builds insert `Pickable` on each block's `Visual` entity in `spawn_block_entity`; a manual `viewport_to_world_2d` hit test was rejected because it would duplicate picking and struggle with UI occlusion. Clicks under UI are blocked by the UI nodes' default `Pickable` behavior, and HUD decorative children already use `Pickable::IGNORE`. The observer ignores clicks while `ControlLeft` is held so Ctrl+drag does not also toggle selection.

3. **Selection state is a resource plus an event.** `ShipDebugSelection { selected: EntityHashSet }` holds the persistent set; an observer writes a `ShipDebugSelected` event and a system applies it to the resource, following the project's event-first communication rule. Alternative: a `Selected` marker component on ships (would need synchronization anyway and splits the state). `EntityHashSet` makes membership checks and removal O(1).

4. **Click toggles membership; Escape clears.** Multiple enemies can be inspected at once. Alternatives considered: click-replaces with Shift-modifier (more to remember), or Ctrl-modifier (conflicts with drag-select).

4b. **Gameplay and dev selection stay independent.** Dev clicks do not emit `EntitySelectEvent` and do not touch `ControlledShip` or `CameraTarget`; Ctrl+drag remains player-only. This avoids wiring debug inspection into command logic.

5. **Text is spawned once and updated on change.** `Text2d`/`TextFont`/`Transform` are inserted when `ShipUIRoot` is created; a system updates `Text2d.0` only when content differs and toggles `Visibility` from the selection set. Alternative: current per-tick `commands.insert` (churn and stale text). Content still reads `AiRuntime.current`, `ShipBehavior`, `ShipStats`, cargo, and per-block ammo dumps.

6. **One gizmo system in `PostUpdate/Present`, gated by `in_state(Game)`.** It draws four families from data already on the selected ships: AI behavior geometry, hull/bounds with a core marker, heading/velocity/force, and turret rays from `TurretState::aim_point` via `BlockOwner`. Alternative: keep gizmos inline in `update_ship_behavior` (cannot isolate or suppress per selection).

7. **Shared geometry lives in an always-compiled module.** Approach/go-around geometry is factored into pure helpers used by both `update_ship_behavior` (production) and the dev renderer. Alternative A: duplicate the math (violates single source of truth). Alternative B: call into a dev-only module from AI code (impossible: production code cannot depend on a dev-gated module).

8. **Module layout follows AGENTS.md.** `content/ship/debug/mod.rs` defines `ShipDebugPlugin`; `component.rs` holds the resource and event; `system.rs` holds observer and rendering systems; `lib.rs` holds cross-module helpers. The plugin is registered from `ContentPlugin` under `#[cfg(feature = "dev")]`. Related systems stay in `SimulationSet::Input` (observer), `FixedUpdate/Present` (text), and `PostUpdate/Present` (gizmos).

## Risks / Trade-offs

- **Stale selection when a ship despawns** → remove the entity on `Despawn`/`Remove` via an observer, and filter reads through a live-entity query as a safety net.
- **Debug text intercepting clicks** → mark the `ShipUIRoot` entity `Pickable::IGNORE` so its `Text2d` never becomes a pick target.
- **Gate migration changes plain debug builds** → debug text disappears without `--features dev`; documented in the proposal and the `performance` spec delta. Rollback is a one-line gate revert.
- **Picking depends on sprite picking and z-order** → no new backend is added; block visuals receive `Pickable` only under the `dev` feature, and the debug UI root is `Pickable::IGNORE` so it cannot steal clicks.
- **Ctrl+drag may emit a click on the release ship** → observer ignores clicks while `ControlLeft` is pressed.
- **Gizmo draw calls scale with selection size** → bounded in practice by the number of selected ships and compiled out of production.

## Migration Plan

1. Add the always-compiled geometry helpers and refactor `update_ship_behavior` to use them, removing its inline gizmo calls.
2. Migrate `ShipUIRoot`, `update_ship_ui`, and their registration from `debug_assertions` to `feature = "dev"`.
3. Add `content/ship/debug/` with selection state, observer, text, and gizmo systems; register the dev-gated plugin in `ContentPlugin`.
4. Update `/docs` specs (UI, controls, scenes, simulation, architecture) with spec-only text.
5. Verify with `cargo dev`; run `cargo dc` only when needed.

Rollback: remove the plugin registration and restore the previous gate and inline gizmos; specs can be reverted from git.

## Open Questions

None blocking. Visual details such as gizmo colors, hull outline shape (rectangle vs chamfered octagon), and line widths are implementation tuning and do not change the requirements.
