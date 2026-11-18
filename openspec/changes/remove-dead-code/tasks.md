## 1. Trim ShipMutation

- [ ] 1.1 In `core/src/content/ship/mutation.rs`, reduce `ShipMutation` to the `DestroyBlock` variant and remove the enum-level `#[allow(dead_code)]`.
- [ ] 1.2 Remove the `DestroyBlocks`, `PlaceBlock`, and `RebuildFromLayout` match arms from `apply_ship_mutations` and simplify the per-ship grouping match.
- [ ] 1.3 Confirm no producer, test, import, or documentation reference to the removed variants remains.

## 2. Remove legacy build-result data

- [ ] 2.1 In `core/src/content/ship/component.rs`, delete `ShipFragment` and the `ShipBuildResult.fragments` field; make `ship`, `stats`, and `bounds` non-optional and drop the `Default` derive.
- [ ] 2.2 In `core/src/content/block/component.rs`, return `Option<ShipBuildResult>` from `build_ship_result`, remove the no-core fragment branch, and remove `ShipFragment` from the import list.
- [ ] 2.3 Update `ShipDesign::spawn_with_id` and `apply_ship_mutations` to bind the `Option<ShipBuildResult>` once instead of matching three fields.
- [ ] 2.4 Rename `core_loss_becomes_fragment` to `core_loss_yields_no_build_result` and assert `build_ship_result(&grid).is_none()`.

## 3. Remove superseded APIs

- [ ] 3.1 Delete `ShipGrid::refresh_cells` and its `#[allow(dead_code)]`.
- [ ] 3.2 Delete `save/design::load_design_file` and its `#[allow(dead_code)]`.

## 4. Gate test-only APIs

- [ ] 4.1 Move `entity_at` and `occupied_cells` into a `#[cfg(test)] impl ShipGrid` inside the tests module of `core/src/content/ship/grid.rs`; remove the production copies and their allowances.
- [ ] 4.2 Add `#[cfg(test)]` to `ShipDesign::new` and remove its allowance.
- [ ] 4.3 Confirm the tests in `core/src/save/battle/mod.rs` still construct designs through `ShipDesign::new`.

## 5. Enforce the guardrail

- [ ] 5.1 Add `dead_code = "deny"` to `[workspace.lints.rust]` in `Cargo.toml`.
- [ ] 5.2 Run `cargo check` and resolve every newly surfaced dead-code site using the same criteria: delete production-unused, `cfg(test)` test-only, implement or expect spec-mandated.
- [ ] 5.3 Confirm no `#[allow(dead_code)]` remains under `core/src`.

## 6. Spec and verification

- [ ] 6.1 Apply the `ship-grid` delta: replace the four-variant mutation requirement with the destruction-only requirement and delete the redundant placement-rejection scenario.
- [ ] 6.2 Run targeted `cargo test` for the ship, block, save, and config modules.
- [ ] 6.3 Run `cargo dev` in the background, smoke-test block destruction and editor placement, then terminate it.
- [ ] 6.4 Confirm `docs/simulation.md` and `docs/architecture.md` remain accurate; update them only if a documented behavior changed.
