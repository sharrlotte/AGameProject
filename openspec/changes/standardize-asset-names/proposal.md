## Why

The asset tree mixes naming conventions (kebab-case, PascalCase, UUID filenames, an uppercase `.PNG` extension) and contains nine orphaned images plus a stray duplicate settings file at the workspace root. The uppercase extension breaks `block-steel.png` on case-sensitive filesystems, and nothing currently prevents new misnamed files from being added.

## What Changes

- Rename `texture/block/block-steel.PNG` to `block-steel.png`; all `.ron` references stay unchanged.
- Remove nine unused images: all three `texture/background/*.png` files and `texture/block/{Nebula1,Nebula2,Nebula3,Stars,block-scrap,ripple}.png`. The background is procedural, so these are dead assets.
- Remove the stray root-level `assets/config/setting.ron`; the active root is `core/assets` and no code change is made to asset-root resolution.
- Add a dev-only `AssetCheckPlugin` that validates asset filenames (lowercase kebab-case, lowercase extension) and emits warnings; it is never registered in production builds.
- Rename two Bevy components to bare nouns per the project naming rule: `DropComponent` -> `Loot`, `ConfigErrorText` -> `ErrorText`.
- Update the three affected spec documents in `/docs` (assets, defects, config) so recorded behavior matches the tree.

## Capabilities

### New Capabilities
- `asset-naming`: asset filename convention, removal of unreferenced assets, and dev-only validation that keeps the tree compliant.

### Modified Capabilities
- None. Config/runtime type splits remain permitted by `code-foundation`, and component renames are internal implementation details that do not change spec behavior.

## Impact

- Filesystem: `core/assets/texture/{block,background}/`, root `assets/config/setting.ron`.
- Code: new `core/src/asset/` module (`mod.rs`, `component.rs`, `lib.rs`, `system.rs`); `core/src/main.rs` dev-only plugin registration.
- Components: `core/src/content/block/component.rs`, `core/src/config/mod.rs`, plus their usage sites.
- Specs: `docs/assets.md`, `docs/defects.md`, `docs/config.md`.
