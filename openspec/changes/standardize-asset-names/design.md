## Context

Assets live in `core/assets` (Bevy default for the `core` crate), but the workspace has a second, stale `assets/config/setting.ron` tracked at the root. Filenames mix conventions: kebab-case (`shoot-spectre.ogg`, `surge-wall-large.png`), PascalCase (`Nebula1.png`, `Stars.png`), an uppercase extension (`block-steel.PNG`), and a UUID (`027078b0-a55c-4139-bd9c-c17709b26df7.png`). Most image paths are indirect: `block.ron` stores strings and `config/block.rs` composes them with `format!("texture/{}", ...)`.

Nine image files are unreferenced. `background.rs` draws the star field with procedural colored sprites and loads no textures, so the entire `texture/background/` directory is dead. `docs/assets.md` already documents most of this, but contains stale `game/` paths and one incorrect "unused" claim (`titanium-wall-large.png` and `shielded-wall.png` are both referenced).

The new project rule requires Bevy component types to be bare nouns, which `DropComponent` and `ConfigErrorText` violate.

## Goals / Non-Goals

**Goals:**
- One filename convention for all shipped assets, with a dev-only guard against regressions.
- A tree containing only assets the game actually loads.
- Remove the stale root-level settings copy without changing asset-root resolution.
- Comply with the bare-noun rule for the two clearly violating Bevy components.

**Non-Goals:**
- Renaming the ~15 already-correct kebab-case files, or changing block IDs in `block.ron`.
- Renaming non-component DTO types such as `BlocksConfig`, `GameConfig`, `MissionDef`, `BlockDef`, or `FeatureConfig`; the earlier decision to rename them was withdrawn, and `code-foundation` explicitly permits config/runtime splits.
- Restructuring the ~30 existing plugins into `mod.rs`; only the new validator follows the full split.
- Reconciling `colaris.png` being shared by the `colaris` and `parallax` blocks (`docs/defects.md` #21 stays open).
- Validating that RON-referenced files exist; the validator checks filenames only.
- Changing `asset_root()` in `config/setting.rs`.

## Decisions

1. **Lowercase kebab-case filenames.** It is already the de-facto standard for the 15 referenced files, so only outliers needed attention. Snake_case was rejected because it would rewrite every working file and every RON path. Because all misnamed files except one are also deleted, the rename set collapses to `block-steel.PNG` -> `block-steel.png`; `block.ron` needs no edit.

2. **Delete nine orphans plus the root settings copy.** Deletion is safe because the background is procedural and the block images have no references. The root copy was introduced accidentally in commit `125734f`. No code change accompanies its deletion: when the game is run from `core/`, both Bevy and `save_setting_on_change` resolve `assets` correctly. A compile-time `CARGO_MANIFEST_DIR` fallback was considered and rejected because it bakes the build machine's path into release binaries.

3. **Dev-only validator with a full four-file module split.** `core/src/asset/` gets `mod.rs` (plugin), `component.rs` (`AssetNameIssue(String)`), `lib.rs` (`is_valid_asset_name(&str) -> bool`, the shared helper), and `system.rs` (`validate_assets`). `#[cfg(feature = "dev")]` sits on the single `add_systems` call, and registration joins the existing dev-only block in `main.rs`. This satisfies the new module-structure rules without adding production code. A CI script (CI currently runs no checks) and a RON asset manifest (would touch the config loader) were considered and deferred.

4. **Names-only validation.** `is_valid_asset_name` checks a lowercase kebab-case stem and a lowercase extension. Missing-file detection was rejected for this change because it would need to parse `block.ron` with the config mirror types and would introduce false positives for intentionally retained art.

5. **Two component renames.** `DropComponent` -> `Loot` (avoiding the `std::mem::Drop` trait name) and `ConfigErrorText` -> `ErrorText`. These are the only Bevy components with literal `Component`/`Config` markers; a wholesale rename of ~50 `*State`/`*Text`/`*Button` components would create collisions such as `AmmoState` vs `Ammo` and was rejected.

6. **Docs updated in place, reduced set.** `docs/assets.md`, `docs/defects.md` (case mismatch resolved, #21 colaris item untouched), and `docs/config.md` (line 189 `DropComponent` -> `Loot`). The other four specs need no change because no type renames occur.

## Risks / Trade-offs

- [Case-only rename on Windows] Git may not register `block-steel.PNG` -> `block-steel.png` in one step → use a two-step rename through a temporary filename and verify with `git status`.
- [Root settings file reappears] Running from the workspace root can recreate `assets/config/setting.ron` → accepted: Bevy would already fail to find `config/base/*.ron` from that directory, so that working directory is unsupported.
- [Validator false positives] Future intentional art with non-conforming names will produce warnings → warnings only, no startup failure; the rule can be amended in the spec.
- [Dev-only code in production] A misplaced `use` or registration could pull the validator into release builds → the `#[cfg(feature = "dev")]` gate is on both the import in `main.rs` and the single registration line.
- [Docs drift] `docs/assets.md` has stale `game/` paths → correct them in the same task that updates the used/unused list.

## Migration Plan

1. Apply the file rename and deletions with git-aware operations; verify the tree with `git status`.
2. Add the validator module and register it in the existing dev-only block in `main.rs`.
3. Rename the two components and fix their usage sites.
4. Update the three spec documents in `/docs`.
5. Run `cargo dc` (dev check) and `cargo dev` in the background to confirm no regressions.

Rollback: all steps are git-tracked file and source changes; revert the change commit to restore the previous tree.

## Open Questions

None. All decisions were resolved during exploration, including the withdrawn Config/Def suffix rule.
