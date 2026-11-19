## 1. Production Ship Inspection State (`content/ship/inspect`)

- [ ] 1.1 Create `core/src/content/ship/inspect/` (`mod.rs`, `component.rs`, `system.rs`, `lib.rs`) defining `InspectedShip(Option<Entity>)`, `ShipSelected { entity }`, `ShipSelectionCleared`, and `ShipInspectPlugin`.
- [ ] 1.2 Implement `ignore_hud_picking`, `on_ship_click` (gating `!team.is_player()` behind `#[cfg(feature = "dev")]` and ignoring `ControlLeft` holds), `clear_selection_on_escape` (before `handle_pause_input`), `apply_selection`, and `remove_despawned_ships` in `SimulationSet::Input` for `SceneState::Game`.
- [ ] 1.3 Register `ShipInspectPlugin` in `core/src/content/ship/mod.rs` (all builds) and update `core/src/content/ship/component.rs` to observe `on_ship_click` on all spawned ships and remove `ShipUIRoot` world-space `Text2d`.
- [ ] 1.4 Update `on_ship_button` in `core/src/scene/game/mod.rs` so clicking a ship in `ShipList` also emits `ShipSelected { entity }`.

## 2. Refactor `ShipDebugPlugin` (`content/ship/debug`)

- [ ] 2.1 Remove `ShipDebugSelection`, `ShipDebugSelected`, `ShipDebugSelectionCleared`, `ShipUIRoot`, and `update_ship_ui` from `core/src/content/ship/debug/`.
- [ ] 2.2 Update `draw_ship_debug_gizmos` to read `Res<InspectedShip>` (only drawing when `Some(entity)`) and keep `ShipDebugPlugin` gated under `#[cfg(feature = "dev")]`.
- [ ] 2.3 Update unit tests in `content/ship/inspect/system.rs` (and `debug/system.rs` if applicable) to cover player selection, toggle, Escape clear, despawn clear, and `dev` vs non-`dev` enemy gating.

## 3. Ship Inspector HUD (`scene/game/inspector`)

- [ ] 3.1 Create `core/src/scene/game/inspector/` (`mod.rs`, `component.rs`, `system.rs`) and mount the bottom-right `ShipInspectorRoot` panel inside `core/src/scene/game/mod.rs` (`DespawnOnExit(SceneState::Game)`).
- [ ] 3.2 Implement structural block-diagram builder triggered only on `InspectedShip` change or `ShipChanged` for the inspected ship, spawning `InspectorBlockNode(block_entity)` `ImageNode`s from `BlockRegistry` (`Feature::Texture` / `Feature::Turret`) with `placement.xform` rotation/flip and scaled grid layout.
- [ ] 3.3 Implement ~80ms in-place refresh system updating `ShipInspectorRoot` visibility, `InspectorBlockNode` `ImageNode.color` from `Health`, and stat bars/text (Hull HP, Power, Cargo, Mobility, Combat, AI Role/Doctrine/Tactic, plus `#[cfg(feature = "dev")]` `Skill`, `Steering`, and `Ammo`/`AmmoState` summaries).
- [ ] 3.4 Add unit tests verifying the block diagram does not rebuild on regular ticks without `InspectedShip` or `ShipChanged` changes, and hides when `InspectedShip` is `None`.

## 4. Verification & Documentation

- [ ] 4.1 Run `cargo test` for both default (`dev`) and `--no-default-features` (production) builds to verify compilation and tests pass.
- [ ] 4.2 Update `/docs` (`architecture.md`, `scenes.md`, `simulation.md`, `ui.md`) to reflect `ship-inspector` and the updated `ship-debug` spec.
