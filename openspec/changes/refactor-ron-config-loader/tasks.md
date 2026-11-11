## 1. Generic Loader Infrastructure (`crates/data`)

- [ ] 1.1 Refactor `game/crates/data/src/loader.rs` to replace `ConfigHandles`, `ConfigAssetStores`, `ActiveConfigs`, `start_loading_configs`, and `check_initial_configs_loaded` with `RonConfig`, `ConfigHandle<T>`, `PendingConfigs`, `ConfigSyncSet`, `AppRonConfigExt::register_ron_config::<T>()`, and generic `start_loading_config::<T>`, `sync_initial_config::<T>` (logging `LoadState::Failed` errors once), `finish_initial_config_load`, and scoped `hot_reload_config::<T>`.
- [ ] 1.2 Add a generic RON parse-test helper (`assert_ron_config_parses::<T>()`) in `game/crates/data/src/loader.rs` that resolves `game/core/assets/<T::PATH>` for any workspace crate, and re-export `RonConfig`, `AppRonConfigExt`, and the test helper from `game/crates/data/src/lib.rs`.
- [ ] 1.3 Decouple `game/crates/data/src/state.rs` from `UpgradesConfig` by keeping `UpgradeLevels` and `UpgradeModifiers` structs and config-free methods (`level`, `increment`) in `data::state` while removing `UpgradesConfig`-dependent calculation methods.

## 2. Feature-Crate Config Migration

- [ ] 2.1 Add `serde.workspace = true` (and `ron.workspace = true` if required by tests) to `Cargo.toml` in `world`, `combat`, `enemies`, `extraction`, and `presentation`.
- [ ] 2.2 Move `ArenaConfig` (`ObstacleDef`) and `PlayerConfig` to `game/crates/sim/world`, implement `RonConfig`, register them in `ArenaPlugin` and `PlayerPlugin`, and add RON parse tests.
- [ ] 2.3 Move `WeaponsConfig` (`WeaponSpecData`) to `game/crates/sim/combat`, implement `RonConfig`, register it in `CombatPlugin`, and add RON parse tests.
- [ ] 2.4 Move `EnemiesConfig` (`EnemyArchetypeData`) to `game/crates/sim/enemies`, implement `RonConfig`, register it in `EnemiesPlugin`, update `ArenaConfig`/`PlayerConfig` imports from `world`, and add RON parse tests.
- [ ] 2.5 Move `ExtractionConfig`, `UpgradesConfig`, and `UpgradesConfig`-dependent upgrade calculation helpers (`max_level`, `next_cost`, `recompute`) to `game/crates/sim/extraction`, implement `RonConfig`, register them in `ExtractionPlugin` and `UpgradesPlugin`, and add RON parse tests.
- [ ] 2.6 Move `VisualsConfig` (`PlayerModelConfig`) to `game/crates/presentation`, implement `RonConfig`, register it in `PlayerVisualsPlugin` (or `VisualsPlugin`), update imports across `camera.rs`, `input.rs`, `menus.rs`, `player_visuals.rs`, and `visuals.rs`, and add RON parse tests.
- [ ] 2.7 Remove `game/crates/data/src/config.rs` and `pub mod config;` from `game/crates/data/src/lib.rs`.

## 3. Documentation & Verification

- [ ] 3.1 Update `docs/07-technical-architecture-save-and-roadmap.md` to reflect generic `register_ron_config` loading and feature-crate config ownership.
- [ ] 3.2 Run `cargo check --workspace` and `cargo test --workspace` and verify all crates compile and all RON parse and simulation tests pass.
