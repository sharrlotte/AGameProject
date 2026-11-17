## Why

The repository accumulates structural drift from the conventions in `AGENTS.md`: a full audit found 26 plugins defined outside `mod.rs`, scattered `#[cfg]` attributes, inline fully-qualified `crate::` paths in production system parameters and bodies, type definitions after functions, a stale asset-root path in the rules themselves, and a duplicate settings file. With the `rewrite-save-system` change now applied, the save layer is compliant and the remaining drift can be cleaned up without overlapping in-flight work.

## What Changes

- **Relocate every plugin into its module's `mod.rs`**: 26 remaining plugins move out of flat `.rs` files and `system.rs` files. Flat plugin files become folders (`scene/editor.rs` → `scene/editor/mod.rs`, `scene/game.rs` → `scene/game/mod.rs`, `scene/background.rs` → `scene/background/mod.rs`, `scene/widget.rs` → `scene/widget/mod.rs`, `scene/notice.rs` → `scene/notice/mod.rs`, `scene/home.rs` → `scene/home/mod.rs`, `scene/pause.rs` → `scene/pause/mod.rs`, `scene/setting.rs` → `scene/setting/mod.rs`, `content/ship/indicator.rs` → `content/ship/indicator/mod.rs`, crate-root `system.rs` → `system/mod.rs`), `content/plugin.rs` folds into `content/mod.rs`, and leaf plugins move from `*/system.rs` into their module's `mod.rs`.
- **Enforce file-role separation**: `system.rs` holds functions only, `component.rs` holds enums/structs/traits/components, and `lib.rs` holds helpers used across modules; `ShipInfo` and `ShipIndex` move out of `content/ship/ai/system.rs` into `component.rs`.
- **Merge `#[cfg]` attributes into the least block**: consolidate the repeated `#[cfg(feature = "dev")]` / `#[cfg(all(debug_assertions, feature = "dev"))]` gates in `main.rs`, `asset/mod.rs`, and the four separate `#[cfg(test)]` items in `content/block/feature/power/grid.rs`.
- **Remove inline fully-qualified `crate::` paths** from function signatures, system parameters, and function bodies (three production sites plus test sites in `content/ship/system.rs`, and test-only sites in `save/battle/mod.rs`, `save/design/lib.rs`, `save/campaign/lib.rs`, `content/ship/ai/component.rs`), replacing them with top-of-file `use` declarations.
- **Restore definition order**: move types declared after the first function back to the top in `content/block/component.rs`, `scene/game.rs`, and `mission/mod.rs`.
- **Correct the asset contract**: update `AGENTS.md` to name `core\assets` as the assets folder and delete the stale, unread duplicate `core/config/setting.ron`.
- **Add regression guards** so future code cannot silently reintroduce these structural violations.
- **BREAKING**: none at runtime. Public module paths for relocatable plugins change only where crate-internal `use` statements must follow the new file locations; no save schema or gameplay behavior changes.

Out of scope: component/type renaming (14 suffixed names, dedicated follow-up change), file and function length splitting (13 files over 300 lines and ~11 long functions, dedicated follow-up change), and the newly added fail-loud/logging rule.

## Capabilities

### New Capabilities
- `code-layout-conventions`: Source-tree conventions that keep module responsibilities focused — plugin placement in `mod.rs`, per-file role separation, minimal `#[cfg]` scope, imported rather than inline crate paths, declaration order, and a single authoritative asset/config location.

### Modified Capabilities
None.

## Impact

- `core/src/scene/`: `editor.rs`, `game.rs`, `home.rs`, `pause.rs`, `setting.rs`, `notice.rs`, `background.rs`, `widget.rs` become `mod.rs`-based modules; `main.rs` imports follow.
- `core/src/content/`: ~18 feature plugins move from `system.rs`/flat files into their module's `mod.rs`; `content/plugin.rs` folds into `content/mod.rs`; `content/ship/ai/system.rs` loses its struct definitions.
- `core/src/system.rs`: becomes `core/src/system/mod.rs` for `DebugAuditPlugin`.
- `core/src/asset/mod.rs`, `core/src/content/block/feature/power/grid.rs`: `#[cfg]` consolidation.
- `core/src/content/ship/system.rs`, `core/src/content/ship/ai/component.rs`, `core/src/save/{battle,design,campaign}/`: inline path cleanup.
- `core/src/content/block/component.rs`, `core/src/scene/game.rs`, `core/src/mission/mod.rs`: declaration reorder.
- `AGENTS.md`: asset path corrected; `core/config/setting.ron` deleted.
- Tests: new structural regression tests; `cargo test` remains the acceptance gate per `code-foundation`.
