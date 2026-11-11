## Context

`game/crates/data/src/loader.rs` currently hardcodes all 7 `.ron` configuration structs (`ArenaConfig`, `PlayerConfig`, `WeaponsConfig`, `EnemiesConfig`, `ExtractionConfig`, `UpgradesConfig`, `VisualsConfig`) across 8 separate places: imports, `ConfigHandles`, `init_asset`, `register_asset_loader`, `init_resource`, `hot_reload_config` system tuples, `start_loading_configs`, `ConfigAssetStores`/`ActiveConfigs` `SystemParam` structs, `check_initial_configs_loaded`, and the `all_ron_config_files_parse_cleanly` test.

At the same time, the workspace crate graph is strictly layered:
- `world` depends on `data`
- `combat`, `enemies`, and `extraction` depend on `data` and `world`
- `presentation` depends on `data`, `world`, `combat`, `enemies`, and `extraction`
- `core` wires all plugins together

Because `data` is the base crate, keeping all config structs inside `data::config` couples `data` to every feature domain, whereas `data` cannot depend on feature crates.

## Goals / Non-Goals

**Goals:**
- Eliminate all per-config duplication and monolithic handle/store/active structs in `data::loader`.
- Enable any feature crate to own its `.ron` config struct(s) and register them via a single `app.register_ron_config::<T>()` call.
- Preserve the existing startup readiness guarantee (`GameState::Loading` waits for all registered configs to populate their `ResMut<T>` resources before transitioning to `GameState::Playing` and emitting `ConfigHotReloadedMessage`).
- Explicitly log `LoadState::Failed` errors if a `.ron` asset fails to load instead of waiting silently.
- Ensure every registered config's `.ron` file has parse test coverage in its owning crate via a shared `data` test helper.

**Non-Goals:**
- Introducing link-time auto-registration crates such as `inventory` or `linkme`, or proc-macro crates.
- Changing the `.ron` asset file paths or schemas in `game/core/assets/config/`.
- Falling back to `Default` and entering `GameState::Playing` when a `.ron` file fails to load (loading remains strict).

## Decisions

### 1. Generic `RonConfig` trait and `AppRonConfigExt::register_ron_config::<T>()`
- **Decision**: Define a `RonConfig` trait in `data::loader` requiring `Asset + Resource<Mutability = Mutable> + Clone + Default + DeserializeOwned + TypePath + Send + Sync + 'static` plus `const PATH: &'static str`. Expose `AppRonConfigExt` with `fn register_ron_config<T: RonConfig>(&mut self) -> &mut Self` on Bevy `App`.
- **Rationale**: Each feature plugin registers its own configs in one call without `data` knowing downstream types and without external dependencies.
- **Alternatives considered**:
  - *Central `macro_rules!` table inside `data`*: Rejected because `data` cannot import types moved into feature crates without creating circular dependencies.
  - *`inventory` auto-registration*: Rejected to avoid adding a link-time global registry dependency when explicit per-plugin registration fits Bevy's `Plugin` model.

### 2. Per-type `ConfigHandle<T>` + `PendingConfigs` loading-gate resource
- **Decision**: Replace `ConfigHandles`, `ConfigAssetStores`, and `ActiveConfigs` with a generic `ConfigHandle<T>(Handle<T>)` resource per config and a shared `PendingConfigs` resource (`Vec` or `HashSet` of pending untyped handles + failed-logged set) initialized in `DataPlugin::build` and `register_ron_config::<T>()`.
- **Rationale**: Any number of feature plugins can register configs in any order; `DataPlugin` only checks whether `PendingConfigs` is empty before transitioning `GameState::Loading -> GameState::Playing`.
- **Failure handling**: During `GameState::Loading`, if `asset_server.load_state(&handle.0)` returns `LoadState::Failed(err)`, log an `error!` once for that handle and keep it pending so `GameState::Playing` is not entered with broken data.
- **Duplicate registration guard**: Guard `register_ron_config::<T>` (e.g., via `world.contains_resource::<ConfigMarker<T>>()` or tracking `TypeId` in `PendingConfigs`) so calling it twice for the same `T` is a safe no-op rather than registering duplicate systems.

### 3. Schedule scoping and system ordering
- **Decision**:
  - `Startup` system `start_loading_config::<T>` loads `T::PATH`, inserts `ConfigHandle<T>`, and pushes the untyped handle into `PendingConfigs`.
  - `Update` systems `sync_initial_config::<T>` run inside `ConfigSyncSet` with `.run_if(in_state(GameState::Loading))`. When `Assets<T>` contains the handle, `*active = asset.clone()` and the handle is removed from `PendingConfigs`.
  - `DataPlugin` adds `finish_initial_config_load` in `Update` `.after(ConfigSyncSet).run_if(in_state(GameState::Loading))` to emit `ConfigHotReloadedMessage` and transition `NextState<GameState>` to `GameState::Playing` once `PendingConfigs` is empty and at least one config (or startup) completed.
  - `hot_reload_config::<T>` runs in `Update` `.before(SimUpdateSet).run_if(not(in_state(GameState::Loading)))` and handles `AssetEvent::Modified` matching `&handle.0`.
- **Rationale**: Prevents initial-load polling from running after startup, avoids racing `finish_initial_config_load` against `sync_initial_config::<T>`, and prevents `hot_reload_config` from double-firing during `GameState::Loading`.

### 4. Feature-crate ownership and `UpgradesConfig` decoupling
- **Decision**: Move each config struct (and its sub-structs) to the crate that owns its domain:
  - `world`: `ArenaConfig`, `ObstacleDef`, `PlayerConfig` (registered in `ArenaPlugin` / `PlayerPlugin`)
  - `combat`: `WeaponsConfig`, `WeaponSpecData` (registered in `CombatPlugin`)
  - `enemies`: `EnemiesConfig`, `EnemyArchetypeData` (registered in `EnemiesPlugin`)
  - `extraction`: `ExtractionConfig`, `UpgradesConfig` (registered in `ExtractionPlugin` / `UpgradesPlugin`)
  - `presentation`: `VisualsConfig`, `PlayerModelConfig` (registered in `PlayerVisualsPlugin` or `VisualsPlugin`)
- **Handling `UpgradeLevels` & `UpgradeModifiers`**: `world`, `combat`, and `enemies` read `UpgradeModifiers` from `data::state`, and `world` does not depend on `extraction`. Therefore `UpgradeLevels` and `UpgradeModifiers` structs (plus config-free `level`/`increment`) remain in `data::state`, while `UpgradesConfig`-dependent logic (`max_level`, `next_cost`, `recompute_upgrade_modifiers` or an `UpgradesExt` trait) moves to `extraction` (used by `extraction` and `presentation`, which already depends on `extraction`).
- **Workspace dependencies**: Add `serde.workspace = true` (and `ron.workspace = true` as `dev-dependencies` if needed by test helper) to feature crates defining config structs.

### 5. Shared RON parse-test helper
- **Decision**: Provide `pub fn assert_ron_config_parses<T: RonConfig>() -> T` (gated or exposed in `data::loader` for tests, using `CARGO_MANIFEST_DIR` / workspace asset path resolution to locate `game/core/assets/<T::PATH>`) so each feature crate validates its own `.ron` file parsing alongside domain assertions without repeating path resolution and file-reading boilerplate.

## Risks / Trade-offs

- **[Risk] Test asset path resolution differs between `data` (`crates/data`) and feature crates (`crates/sim/world`, etc.)** → Mitigation: Have the test helper locate `game/core/assets` relative to `env!("CARGO_MANIFEST_DIR")` by walking ancestor directories (or checking `core/assets` vs `../core/assets` vs `../../core/assets`) so any workspace crate can invoke it reliably.
- **[Risk] `CoreDataPlugin` vs feature plugin `Startup` ordering** → Mitigation: Initialize `PendingConfigs` in `DataPlugin::build` (and idempotently via `init_resource` in `register_ron_config::<T>`), and ensure `finish_initial_config_load` checks a readiness flag or runs in `Update` after `Startup` has populated `PendingConfigs`.
- **[Risk] Existing unit tests in `world`, `combat`, `enemies`, `extraction` calling `.init_resource::<...>()` directly** → Mitigation: Keep `Default` implementations on all config structs and keep `UpgradeModifiers` in `data` so minimal unit tests continue to compile cleanly after import updates.
- **[Risk] Missing `/docs` update** → Mitigation: Include updating `docs/07-technical-architecture-save-and-roadmap.md` in `tasks.md`.
