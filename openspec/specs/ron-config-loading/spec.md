# ron-config-loading Specification

## Purpose
Generic `.ron` configuration registration, startup readiness gating, scoped hot-reloading, feature-crate ownership, and parse verification.

## Requirements

### Requirement: Generic RON config registration
The `data` crate SHALL provide a `RonConfig` trait and an `AppRonConfigExt::register_ron_config::<T>()` extension method that registers a configuration type's asset, `RonAssetLoader<T>`, active `Resource`, startup asset load, initial-load sync system, and hot-reload system without per-type boilerplate structs.

#### Scenario: Registering a config type via a single call
- **WHEN** a plugin calls `app.register_ron_config::<T>()` for a type `T` implementing `RonConfig`
- **THEN** the `App` initializes `Assets<T>`, registers `RonAssetLoader::<T>`, initializes `Res<T>` with `T::default()`, schedules loading of `T::PATH` on `Startup`, and schedules initial sync and hot-reload systems for `T`

#### Scenario: Duplicate registration is idempotent
- **WHEN** `app.register_ron_config::<T>()` is called more than once for the same type `T`
- **THEN** the second call does not register duplicate startup, sync, or hot-reload systems or duplicate pending handles

### Requirement: Unified initial loading gate with failure logging
`DataPlugin` SHALL track all registered configs in a generic `PendingConfigs` resource during `GameState::Loading`, populate each `ResMut<T>` before leaving `GameState::Loading`, log an error if any asset fails to load, and transition to `GameState::Playing` only when all registered configs are ready.

#### Scenario: All registered configs load successfully
- **WHEN** all registered configs finish loading into `Assets<T>` during `GameState::Loading`
- **THEN** each active `ResMut<T>` holds the parsed `.ron` asset value, `PendingConfigs` becomes empty, a single `ConfigHotReloadedMessage` is emitted, and `NextState<GameState>` transitions to `GameState::Playing`

#### Scenario: A config asset fails to load
- **WHEN** a registered config's `LoadState` becomes `LoadState::Failed` during `GameState::Loading`
- **THEN** the system logs an error identifying the failed config path/error once and keeps the application in `GameState::Loading`

### Requirement: Scoped config hot-reloading
The loader SHALL run initial synchronization only while in `GameState::Loading` and SHALL run `hot_reload_config::<T>` before `SimUpdateSet` only when not in `GameState::Loading`.

#### Scenario: Modifying a RON config while playing or in menus
- **WHEN** an `AssetEvent::Modified` event is emitted for a registered config handle outside `GameState::Loading`
- **THEN** the corresponding `ResMut<T>` is updated with the new asset value and `ConfigHotReloadedMessage` is emitted before `SimUpdateSet`

### Requirement: Feature-crate config ownership
Each feature crate SHALL own and register its domain configuration structs rather than centralizing them in `data`:
- `world` owns `ArenaConfig` (`ObstacleDef`) and `PlayerConfig`
- `combat` owns `WeaponsConfig` (`WeaponSpecData`)
- `enemies` owns `EnemiesConfig` (`EnemyArchetypeData`)
- `extraction` owns `ExtractionConfig` and `UpgradesConfig` (along with `UpgradesConfig`-dependent upgrade calculation helpers)
- `presentation` owns `VisualsConfig` (`PlayerModelConfig`)

Shared upgrade state types (`UpgradeLevels`, `UpgradeModifiers`) SHALL remain in `data::state` without depending on `UpgradesConfig`.

#### Scenario: Feature crates compile without circular dependencies
- **WHEN** each feature crate defines and registers its own config structs and `data` defines only generic loader infrastructure and shared state/messages
- **THEN** the entire workspace builds cleanly with no dependency from `data` to any feature crate

### Requirement: Per-config RON parse verification
The `data` crate SHALL provide a generic RON parse-test helper that resolves `game/core/assets/<T::PATH>` and deserializes `T`, and each owning feature crate SHALL verify its `.ron` config file(s) parse cleanly in unit tests.

#### Scenario: Running workspace tests validates all RON config files
- **WHEN** `cargo test` is executed across the workspace
- **THEN** every `.ron` file in `game/core/assets/config/` is deserialized into its owning config struct and passes both parse validation and domain assertions
