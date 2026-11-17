## 1. Asset Tree Cleanup

- [ ] 1.1 Rename `core/assets/texture/block/block-steel.PNG` to `block-steel.png` using a two-step git-aware rename so the case change is tracked on Windows; leave `block.ron` unchanged.
- [ ] 1.2 Delete the three unreferenced background images: `027078b0-a55c-4139-bd9c-c17709b26df7.png`, `background1.png`, `background2.png` under `core/assets/texture/background/`.
- [ ] 1.3 Delete the six unreferenced block images: `Nebula1.png`, `Nebula2.png`, `Nebula3.png`, `Stars.png`, `block-scrap.png`, `ripple.png` under `core/assets/texture/block/`.
- [ ] 1.4 Delete the stale root-level `assets/config/setting.ron` without modifying `asset_root()` in `core/src/config/setting.rs`.
- [ ] 1.5 Verify with `git status` that the rename and all deletions are recorded, and confirm no remaining file references the removed names.

## 2. Dev-Only Asset Validator

- [ ] 2.1 Create `core/src/asset/mod.rs` defining `AssetCheckPlugin` with `use bevy::prelude::*`, module declarations, and `#[cfg(feature = "dev")]` on the single `add_systems(Startup, validate_assets)` call.
- [ ] 2.2 Create `core/src/asset/component.rs` defining `AssetNameIssue(String)` with no prefix or suffix.
- [ ] 2.3 Create `core/src/asset/lib.rs` defining the shared helper `is_valid_asset_name(&str) -> bool` that accepts a lowercase kebab-case stem and a lowercase extension.
- [ ] 2.4 Create `core/src/asset/system.rs` defining `validate_assets`, which walks `core/assets` and emits a `warn!` per filename rejected by `is_valid_asset_name`, without failing startup.
- [ ] 2.5 Add `mod asset;` to `core/src/main.rs` and register `AssetCheckPlugin` inside the existing `#[cfg(feature = "dev")]` block alongside `RemotePlugin` and `BrpExtrasPlugin`.

## 3. Bevy Component Noun Renames

- [ ] 3.1 Rename `DropComponent` to `Loot` in `core/src/content/block/component.rs` and update all usage sites.
- [ ] 3.2 Rename `ConfigErrorText` to `ErrorText` in `core/src/config/mod.rs` and update all usage sites.

## 4. Spec Document Updates

- [ ] 4.1 Update `docs/assets.md`: correct the root to `core/assets`, document the lowercase kebab-case convention and dev validator, replace the outdated unused-assets list with the removal result, and fix the stale `titanium-wall-large`/`shielded-wall` claim.
- [ ] 4.2 Update `docs/defects.md`: mark the `block-steel.PNG` case mismatch resolved while leaving the `colaris.png` item in #21 open.
- [ ] 4.3 Update `docs/config.md` line 189: replace `DropComponent` with `Loot`.

## 5. Verification

- [ ] 5.1 Run `cargo dc` and resolve any compile or lint errors from the renames and new module.
- [ ] 5.2 Run `cargo dev` in the background, confirm a clean startup with no filename warnings, then terminate the process manually.
