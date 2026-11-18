## 1. Shared geometry and gate migration

- [x] 1.1 Add always-compiled pure helpers in `content/ship` for `Approach` and `GoAround` target/ring geometry, with unit tests for computed positions
- [x] 1.2 Refactor `update_ship_behavior` to consume the shared helpers and remove its inline gizmo calls without changing steering behavior
- [x] 1.3 Migrate `ShipUIRoot`, its spawn block, `update_ship_ui`, and `ShipPlugin` registration from `#[cfg(debug_assertions)]` to `#[cfg(feature = "dev")]`, consolidating gates

## 2. Selection state and input

- [x] 2.1 Create `content/ship/debug/` with `mod.rs`, `component.rs`, `system.rs`, and `lib.rs`
- [x] 2.2 Define `ShipDebugSelection { selected: EntityHashSet }` and a `ShipDebugSelected` event; initialize them in `ShipDebugPlugin`
- [x] 2.3 Add a dev-only `On<PointerClick>` observer on ship roots that toggles membership for any team, ignores clicks while `ControlLeft` is held, and emits `ShipDebugSelected`
- [x] 2.4 Apply `ShipDebugSelected` to the resource and clear the selection on Escape
- [x] 2.5 Remove despawned ships from the selection via a cleanup observer and guard reads against stale entities
- [x] 2.6 Unit-test toggling, Escape clear, and independence from `EntitySelectEvent`

## 3. Selection-scoped debug text

- [x] 3.1 Spawn `Text2d`, `TextFont`, and `Transform` once on `ShipUIRoot` (hidden by default) and mark the root `Pickable::IGNORE`
- [x] 3.2 Replace `update_ship_ui` with a selection-scoped updater that changes `Text2d` content only on change and toggles `Visibility` for selected ships
- [x] 3.3 Register the text system in `FixedUpdate/Present`, gated by `feature = "dev"` and `in_state(SceneState::Game)`

## 4. Gizmo renderer

- [x] 4.1 Implement `draw_ship_debug_gizmos` in `content/ship/debug/system.rs`, iterating only ships in `ShipDebugSelection`
- [x] 4.2 Draw AI behavior geometry via the shared helpers (approach line/target circle, go-around ring/next point)
- [x] 4.3 Draw hull/bounds outline and core-position marker from `BoundingBox`, `Ship.core_position`, and `Transform`
- [x] 4.4 Draw heading, linear velocity, and applied force from `Transform::local_y()`, `Velocity`, and `ExternalForce`
- [x] 4.5 Draw turret aim rays from `TurretState::aim_point` for turret blocks whose `BlockOwner.body` is selected
- [x] 4.6 Register the gizmo system in `PostUpdate/Present`, gated by `feature = "dev"` and `in_state(SceneState::Game)`

## 5. Wiring and documentation

- [x] 5.1 Register `ShipDebugPlugin` from `ContentPlugin` under `#[cfg(feature = "dev")]`
- [x] 5.2 Update `/docs` spec text in `ui.md`, `controls.md`, `scenes.md`, `simulation.md`, and `architecture.md` for the new gate, click selection, and selection-scoped overlay
- [x] 5.3 Verify with `cargo dev`: click an enemy to select/deselect, confirm text and all four gizmo families appear only for selected ships, and confirm Ctrl+drag still behaves as before
- [x] 5.4 Run `cargo dc` if a targeted check is needed after integration
