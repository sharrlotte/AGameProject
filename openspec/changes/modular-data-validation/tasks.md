## 1. Shared Validation Framework

- [ ] 1.1 Create `core/src/validate/mod.rs` with `pub trait Validate` and `pub mod check` assertions (`positive`, `non_negative`, `range_ordered`, `bounded_int`, `finite`)
- [ ] 1.2 Add unit tests for all `check::*` functions in `core/src/validate/mod.rs`

## 2. Modular Config System

- [ ] 2.1 Split `core/src/config/game.rs` into `core/src/config/game/game.rs` and `core/src/config/game/fleet.rs` with `impl Validate for GameConfig`
- [ ] 2.2 Split `core/src/config/block.rs` into `core/src/config/block/def.rs` and `core/src/config/block/feature/` (`turret.rs`, `shield.rs`, `thruster.rs`, `power.rs`, `ammo.rs`, `texture.rs`, `storage.rs`)
- [ ] 2.3 Implement `Validate` and clean conversion methods inside each respective block feature module
- [ ] 2.4 Implement `Validate` for `MinimapConfig` and `AiConfig` in their respective modules
- [ ] 2.5 Update `core/src/config/loader.rs` to validate assets implementing `Validate<Context = ()>` automatically at decode time
- [ ] 2.6 Simplify `check_config_ready` in `core/src/config/mod.rs` to rely on loader decode-time validation and remove duplicate manual checks

## 3. Modular Save Snapshot System

- [ ] 3.1 Split `core/src/save/world/snapshot/component.rs` into dedicated single-file data modules: `ship.rs`, `block.rs`, `debris.rs`, `design.rs`
- [ ] 3.2 Implement `Validate` for `ShipSnapshot`, `BlockSnapshot`, `DebrisSnapshot`, and `ShipDesign` in their respective files
- [ ] 3.3 Create `core/src/save/world/snapshot/sanitize.rs` for transient value clamping/normalization (charge, cooldown, remaining lifetime)
- [ ] 3.4 Update snapshot decode and apply callers to use the modular snapshot types and validation paths

## 4. Documentation and Verification

- [ ] 4.1 Update `/docs/config.md`, `/docs/save.md`, and `/docs/architecture.md` (spec only, no notes)
- [ ] 4.2 Run targeted unit tests (`cargo test`) for config, validate, and save modules
- [ ] 4.3 Run `cargo dev` in the background and verify boot, hot-reload failure reporting, and save loading
