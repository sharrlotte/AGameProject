## 1. Regression guard

- [ ] 1.1 Add `core/tests/conventions.rs` using only `std::fs` and `CARGO_MANIFEST_DIR` to scan `core/src/**/*.rs` and assert: every `impl Plugin` is in `mod.rs`, `system.rs` declares no types, same-gate `#[cfg]` items are grouped, no inline `crate::` path in signatures/parameters/bodies, and types precede functions with tests last.
- [ ] 1.2 Run `cargo test --test conventions` and record the baseline failures as the work list for the following sections.

## 2. Scene module relocation

- [ ] 2.1 Convert `core/src/scene/editor.rs` into `core/src/scene/editor/mod.rs`, preserve the existing `pub use` re-exports, and declare `ShipEditorPlugin` in it.
- [ ] 2.2 Convert `scene/game.rs`, `scene/home.rs`, `scene/pause.rs`, `scene/setting.rs`, `scene/notice.rs`, `scene/background.rs`, and `scene/widget.rs` into `mod.rs`-based folders with their plugins, keeping `pub use` re-exports so existing import paths resolve.
- [ ] 2.3 Move the type declarations in `scene/game.rs` (`ShipList`, `CargoHudText`, `SelectablePanel`, `GameScenePlugin`) above the first function.
- [ ] 2.4 Update `main.rs` imports to follow the converted scene module paths.

## 3. Content module relocation

- [ ] 3.1 Move leaf plugins from `content/*/system.rs` into their module's `mod.rs` for bullet, damage, death, item, lifetime, block, selectable, block/feature, block/radar, and block/feature/{ammo,turret,thruster}.
- [ ] 3.2 Move plugins in `content/ship/system.rs`, `content/ship/mutation.rs`, `content/ship/indicator.rs`, and `content/ship/ai/system.rs` into `content/ship/mod.rs` and `content/ship/ai/mod.rs`.
- [ ] 3.3 Fold `content/plugin.rs` into `content/mod.rs` and update its `pub use` list.
- [ ] 3.4 Convert crate-root `core/src/system.rs` into `core/src/system/mod.rs`, keeping `DebugAuditPlugin` behind its debug gate, and update `main.rs` imports.
- [ ] 3.5 Move `ShipInfo` and `ShipIndex` out of `content/ship/ai/system.rs` into `content/ship/ai/component.rs` with the other types.
- [ ] 3.6 Verify `cargo test --test conventions` no longer reports plugin-placement or `system.rs` type violations.

## 4. Conditional compilation consolidation

- [ ] 4.1 Merge the repeated `#[cfg(feature = "dev")]` imports and the three dev/debug plugin-registration blocks in `main.rs` into one block per scope.
- [ ] 4.2 Merge the five `#[cfg(feature = "dev")]` items in `asset/mod.rs` into one block.
- [ ] 4.3 Merge the four separate `#[cfg(test)]` items in `content/block/feature/power/grid.rs` into one test block.

## 5. Imports and declaration order

- [ ] 5.1 Replace the inline `crate::content::ship::ai::AiRuntime`, `crate::constant::LAYER_SHIP_UI`, and `crate::control::system::toggle_control_mode` paths in `content/ship/system.rs` with top-of-file `use` declarations.
- [ ] 5.2 Replace inline `crate::` paths in test modules (`content/ship/ai/component.rs`, `save/battle/mod.rs`, `save/design/lib.rs`, `save/campaign/lib.rs`) with crate-local `use` declarations.
- [ ] 5.3 Move the type declarations that currently follow the first function in `content/block/component.rs` and `mission/mod.rs` to the top of each file, leaving tests last.
- [ ] 5.4 Verify `cargo test --test conventions` no longer reports inline-path or ordering violations.

## 6. Asset location contract

- [ ] 6.1 Change the `Assets folder` entry in `AGENTS.md` from `game\assets` to `core\assets`.
- [ ] 6.2 Delete the stale, unread `core/config/setting.ron` duplicate, leaving `core/assets/config/setting.ron` as the only settings source.
- [ ] 6.3 Confirm `core/assets/config/setting.ron` loads through `AssetServer` and that `docs/config.md` still documents the `BEVY_ASSET_ROOT`/CWD write-back caveat.

## 7. Verification and specification record

- [ ] 7.1 Run `cargo test` and confirm the full suite passes, including the new conventions test.
- [ ] 7.2 Run `cargo dev` in the background per the `AGENTS.md` testing rule, smoke-check that all scenes still register, then terminate the process.
- [ ] 7.3 Update `docs/architecture.md` to describe the `mod.rs`-based plugin layout and per-file roles (specification only, no informal notes), per the `AGENTS.md` documentation rule.
