## Context

`AGENTS.md` defines the project's structural conventions, but a repo-wide audit found systematic drift: 26 plugins defined outside `mod.rs`, repeated `#[cfg]` attributes that could be merged, three production inline `crate::` paths in `content/ship/system.rs`, type definitions after the first function in three files, an asset path in `AGENTS.md` that does not exist, and a duplicate `core/config/setting.ron` that no reader uses. The applied `rewrite-save-system` change already made `save/` and its three sub-plugins compliant, so it is no longer a competing editor of the same files. This change addresses only the strict structural rules; permissive length rules, type renaming, and the newly added fail-loud rule are left to separate changes.

## Goals / Non-Goals

**Goals:**
- Make every plugin live in its module's `mod.rs`, with `system.rs` holding functions only and `component.rs` holding types.
- Consolidate `#[cfg]` attributes to the smallest enclosing block.
- Replace inline `crate::` paths in signatures, system parameters, and bodies with top-of-file `use` declarations.
- Restore declaration order (types, then functions, then tests).
- Make the asset path in `AGENTS.md` match the repository and remove the stale duplicate settings file.
- Add regression guards so the conventions cannot silently regress.

**Non-Goals:**
- Renaming the 14 suffixed types (separate follow-up change).
- Splitting the 13 files over 300 lines or the ~11 functions over 100 lines (separate follow-up change).
- Auditing or fixing silent failures under the newly added fail-loud rule.
- Changing runtime behavior, save schema, or gameplay values.

## Decisions

### 1. Literal plugin placement, applied to every plugin
- **Choice**: Every `impl Plugin` moves into its module's `mod.rs`. Flat plugin files become folders (`scene/game.rs` → `scene/game/mod.rs`, `scene/editor.rs` → `scene/editor/mod.rs`, `content/ship/indicator.rs` → `content/ship/indicator/mod.rs`, crate-root `system.rs` → `system/mod.rs`, etc.); `content/plugin.rs` folds into `content/mod.rs`; leaf plugins leave `*/system.rs`.
- **Alternatives**: (a) scene-level plugins only, leaving leaf plugins in `system.rs`; (b) amend `AGENTS.md` to match current practice. Both were rejected because the rule is explicit and the user chose the literal reading.
- **Rationale**: One unambiguous convention is easier to guard and reason about than a two-tier exception.

### 2. Preserve public paths with `pub use` re-exports
- **Choice**: `mod.rs` keeps `pub use` of its sibling modules' public items, so existing paths such as `crate::content::bullet::BulletPlugin` and `crate::scene::editor::ShipEditorPlugin` remain unchanged after the file moves.
- **Alternatives**: Update every call site to a new path. Rejected as needless churn.
- **Rationale**: Keeps the diff local to each module and avoids touching `main.rs` imports beyond what the folder conversion requires.

### 3. File-role separation follows the already-compliant save layout
- **Choice**: Use the `rewrite-save-system` result as the reference pattern: `mod.rs` owns the plugin and module declarations, `system.rs` owns functions, `component.rs` owns enums/structs/traits/components, `lib.rs` owns helpers used by sibling modules. `ShipInfo` and `ShipIndex` move from `content/ship/ai/system.rs` to `content/ship/ai/component.rs`.
- **Alternatives**: Leave helper structs in `system.rs`. Rejected because it violates the rule and mixes data with behavior.
- **Rationale**: Reuses a pattern already proven in the applied work instead of inventing a new one.

### 4. Merge `#[cfg]` into one block per scope
- **Choice**: Combine the repeated `#[cfg(feature = "dev")]` imports and plugin registrations in `main.rs` and `asset/mod.rs`, and the four separate `#[cfg(test)]` items in `content/block/feature/power/grid.rs`, into one block per scope. Debug-only plugin registration stays behind its existing `debug_assertions` guards.
- **Alternatives**: Per-item attributes. Rejected as harder to read and the exact pattern the rule calls out.
- **Rationale**: Reduces attribute noise without widening compilation gates.

### 5. Imported paths; module-level macro invocations stay
- **Choice**: Replace inline `crate::…` in signatures, system parameters, and bodies with top-of-file `use`. Module-level `crate::ron_document!(…)` invocations in `save/{battle,campaign,design}` are left alone because the rule scopes the prohibition to signatures, system parameters, and function bodies.
- **Alternatives**: Convert macro invocations too. Rejected as an over-reading of the rule.
- **Rationale**: Follows the rule's exact wording and keeps the diff bounded.

### 6. Asset contract: correct the doc, delete the stale copy
- **Choice**: Change `AGENTS.md` to name `core\assets` and delete `core/config/setting.ron`. The `BEVY_ASSET_ROOT`/CWD write-back caveat documented in `docs/config.md:46,176` stays as-is.
- **Alternatives**: Move assets to `game\assets` and update `AssetPlugin` plus three `include_str!` paths; or leave the doc stale. Both rejected as unnecessary runtime churn versus a one-line doc fix.
- **Rationale**: The duplicate is provably unread — `AssetServer` loads `core/assets/config/setting.ron` and `asset_root()` resolves to a CWD-relative `assets` directory.

### 7. Regression guards as a test-only integration test
- **Choice**: Add `core/tests/conventions.rs` using `std::fs` and `CARGO_MANIFEST_DIR` to scan `src/**/*.rs` and assert the structural invariants. No new dependency; test-only, so no production code is added.
- **Alternatives**: Clippy lint config (cannot express plugin placement or file roles), or a shell script in `/tools` (not run by `cargo test`). Rejected as weaker gates.
- **Rationale**: `code-foundation` already makes `cargo test` the acceptance gate, so a test is the natural enforcement point.

## Risks / Trade-offs

- **[Large mechanical diff across ~40 files]** → land per module in separate commits; `pub use` re-exports keep call sites unchanged; run `cargo test` after each batch rather than once at the end.
- **[Git rename detection on flat-file → folder moves]** → use `git mv` so history is preserved; verify with `git status` after each conversion.
- **[Source-scanning test brittleness]** → keep matching patterns narrow (declaration keywords and `impl Plugin` only); if a future false positive appears, adjust the pattern rather than exempting source files.
- **[Overlap with the follow-up rename change]** → sequence this change first; the rename change touches `content/block/component.rs` and `scene/editor/mod.rs`, which this change also edits.
- **[Asset doc fix does not repair runtime write-back path]** → explicitly documented as a known limitation in `docs/config.md`; changing it is out of scope.
