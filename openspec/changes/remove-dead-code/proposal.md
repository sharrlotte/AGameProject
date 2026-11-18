## Why

`core` is a binary crate, so `pub` items have no external users. Eight `#[allow(dead_code)]` attributes currently hide APIs that are either superseded, test-only, or specified but not yet implemented:

- `content/ship/grid.rs`: `entity_at`, `occupied_cells`, `refresh_cells`
- `content/ship/component.rs`: `ShipFragment`, `ShipBuildResult.fragments`, `ShipDesign::new`
- `content/ship/mutation.rs`: `ShipMutation::{DestroyBlocks, PlaceBlock, RebuildFromLayout}`
- `save/design/lib.rs`: `load_design_file`

The suppressions hide real drift: `ShipFragment` is already forbidden by `code-foundation`, the placement mutation variants have no producer and `PlaceBlock` would bypass overlap validation, and two functions have no callers at all.

## What Changes

- Delete `ShipGrid::{entity_at, occupied_cells, refresh_cells}` from production. The two query helpers move to a test-local `impl ShipGrid`; `refresh_cells` has no caller and is superseded by `insert_block`/`remove_block`/`rebuild_from_layout`.
- Delete `save/design::load_design_file`; designs are read through `read_designs_dir`/`load_or_seed_designs`.
- Delete `ShipFragment` and `ShipBuildResult.fragments`. `ShipBuildResult` keeps its name but its `ship`, `stats`, and `bounds` fields become non-optional; `BlockRegistry::build_ship_result` returns `Option<ShipBuildResult>`.
- Trim `ShipMutation` to `DestroyBlock`. `DestroyBlocks`, `PlaceBlock`, and `RebuildFromLayout` have no producers, and `PlaceBlock` would not reject overlap.
- Gate `ShipDesign::new` behind `#[cfg(test)]`.
- Add `dead_code = "deny"` to `[workspace.lints.rust]` so new dead code fails the build.
- Update the `ship-grid` mutation requirement to destruction-only and remove the redundant placement-rejection scenario; the overlap guarantee already lives under "Spawn and editor validate ship layouts".
- Rename `core_loss_becomes_fragment` to `core_loss_yields_no_build_result` and assert `build_ship_result` returns `None`.

Out of scope: adding runtime placement, repair, or multi-block destruction; editor design-grid changes; renaming `ShipBuildResult`; enabling `clippy::allow_attributes`; save schema, gameplay, or asset changes.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `ship-grid`: the mutation requirement narrows to runtime block destruction; placement overlap remains governed by spawn and editor validation.

## Impact

- `core/src/content/ship/{grid.rs, component.rs, mutation.rs}`
- `core/src/content/block/component.rs` (`build_ship_result`, `ShipFragment` import)
- `core/src/save/design/lib.rs`
- `Cargo.toml` (`dead_code = "deny"`)
- `openspec/specs/ship-grid/spec.md` via delta
- No save schema, gameplay behavior, asset, or documentation changes.
