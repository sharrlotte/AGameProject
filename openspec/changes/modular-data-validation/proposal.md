## Why

Config and save validation logic is currently coupled with struct definitions, serialization attributes, and entity conversion inside 400+ line monolithic files (`core/src/config/block.rs`, `core/src/save/world/snapshot/component.rs`). This violates separation of concerns, creates repetitive validation boilerplate across subsystems, slows incremental compilation, and allows unvalidated data to leak across systems because validation is manually invoked in disparate ad-hoc passes.

Refactoring each kind of data into its own dedicated file and standardizing validation around a zero-dependency `Validate` trait provides maximum separation, instant decode-time safety, faster compilation, and seamless future expansion without external dependencies.

## What Changes

- Introduce a shared, zero-dependency validation contract (`core/src/validate/mod.rs`) providing `trait Validate` and composable assertions (`check::positive`, `check::non_negative`, `check::range_ordered`, etc.).
- Refactor `RonAssetLoader` to automatically invoke `Validate::validate` on decode, rejecting invalid `.ron` files at the asset loading boundary with precise file and reason details.
- Split `core/src/config/block.rs` into dedicated single-file data modules under `core/src/config/block/feature/` (`turret.rs`, `shield.rs`, `thruster.rs`, `power.rs`, `ammo.rs`, `texture.rs`, `storage.rs`) and `core/src/config/block/def.rs`, keeping each file focused on its schema, validation, and conversion.
- Split `core/src/config/game.rs` into `game.rs` and `fleet.rs`.
- Split `core/src/save/world/snapshot/component.rs` into dedicated single-file data modules (`ship.rs`, `block.rs`, `debris.rs`, `design.rs`), moving structural validation into `impl Validate` and transient field normalization into `sanitize.rs`.
- Update documentation in `/docs` to reflect the modular data layout and decode-time validation contract (spec only, no notes).

## Capabilities

### New Capabilities
- `modular-data-definitions`: Establishes the single-file-per-data-kind structure across configs and snapshots, ensuring each data kind encapsulates its schema, validation, and conversion in a dedicated file under 150 lines.

### Modified Capabilities
- `config-validation`: Shifts config validation from ad-hoc secondary game systems into standard `Validate` implementations triggered automatically during `RonAssetLoader` asset decoding.
- `save-integrity`: Modularizes snapshot definitions into dedicated files, standardizing structural validation through `Validate` while isolating transient value clamping in `sanitize.rs`.

## Impact

- Affected code: `core/src/config/**`, `core/src/save/world/snapshot/**`, `core/src/validate/**`, `core/src/content/block/**`.
- Dependencies: Zero new external crate dependencies.
- Build performance: Improves incremental compilation by decoupling feature configs from a central monolith.
