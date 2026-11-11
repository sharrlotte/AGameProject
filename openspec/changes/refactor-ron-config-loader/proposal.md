## Why

`game/crates/data/src/loader.rs` currently duplicates boilerplate and requires manual registration across 8 separate places (imports, `ConfigHandles`, `init_asset`, `register_asset_loader`, `init_resource`, `ConfigAssetStores`, `ActiveConfigs`, `check_initial_configs_loaded`, `hot_reload_config`, and RON parse tests) for every `.ron` configuration struct. Moving to a generic registration API in `data` and letting each feature crate own its own config structs removes this duplication, prevents cross-crate coupling as more configs are added, and aligns with feature-based plugin modularity.

## What Changes

- Replace the hardcoded multi-config structs and systems in `data::loader` (`ConfigHandles`, `ConfigAssetStores`, `ActiveConfigs`, `start_loading_configs`, `check_initial_configs_loaded`) with a generic `RonConfig` trait, `AppRonConfigExt::register_ron_config::<T>()` extension method, and a generic `PendingConfigs` loading-gate tracker inside `DataPlugin`.
- Add explicit `LoadState::Failed` error logging during `GameState::Loading` while keeping the loading gate strict (staying in `GameState::Loading` if any registered config fails to load).
- Scope initial config synchronization to `GameState::Loading` and keep generic `hot_reload_config::<T>` active outside `GameState::Loading` (before `SimUpdateSet`).
- **BREAKING**: Move all 7 configuration structs and their sub-structs out of `data::config` into their owning feature crates, with each feature plugin calling `app.register_ron_config::<T>()`:
  - `ArenaConfig` (`ObstacleDef`) and `PlayerConfig` -> `world`
  - `WeaponsConfig` (`WeaponSpecData`) -> `combat`
  - `EnemiesConfig` (`EnemyArchetypeData`) -> `enemies`
  - `ExtractionConfig` and `UpgradesConfig` -> `extraction`
  - `VisualsConfig` (`PlayerModelConfig`) -> `presentation`
- Keep `UpgradeLevels` and `UpgradeModifiers` structs in `data::state` (shared by `world`, `combat`, `enemies`, `extraction`, and `presentation`) while moving the `UpgradesConfig`-dependent calculation methods (`max_level`, `next_cost`, `recompute`) into `extraction` so `data` has no dependency on `UpgradesConfig`.
- Provide a generic RON parse-test helper in `data` so each feature crate verifies its own `.ron` file(s) alongside domain invariant assertions.
- Update `docs/07-technical-architecture-save-and-roadmap.md` to reflect generic config registration and feature-owned config structs.

## Capabilities

### New Capabilities
- `ron-config-loading`: Generic `.ron` configuration registration (`RonConfig` trait, `AppRonConfigExt::register_ron_config`), unified `GameState::Loading` readiness gate with failure logging, hot-reload syncing, and feature-crate config ownership.

### Modified Capabilities
- None (no existing specs in `openspec/specs/`).

## Impact

- **`crates/data`**: `loader.rs` refactored to generic infrastructure + test helper; `config.rs` removed (or emptied); `state.rs` decoupled from `UpgradesConfig`; `lib.rs` exports generic config registration API.
- **`crates/sim/world`**: Gains `ArenaConfig`, `ObstacleDef`, `PlayerConfig`; registers them in `ArenaPlugin` and `PlayerPlugin` + parse tests.
- **`crates/sim/combat`**: Gains `WeaponsConfig`, `WeaponSpecData`; registers in `CombatPlugin` + parse/domain tests.
- **`crates/sim/enemies`**: Gains `EnemiesConfig`, `EnemyArchetypeData`; registers in `EnemiesPlugin` + parse tests; imports `ArenaConfig` and `PlayerConfig` from `world`.
- **`crates/sim/extraction`**: Gains `ExtractionConfig`, `UpgradesConfig`, and upgrade calculation helpers; registers in `ExtractionPlugin` and `UpgradesPlugin` + parse tests.
- **`crates/presentation`**: Gains `VisualsConfig`, `PlayerModelConfig`; registers in `PlayerVisualsPlugin` (or `VisualsPlugin`) + parse tests; updates imports for `ArenaConfig`, `PlayerConfig`, `EnemiesConfig`, `UpgradesConfig`, and upgrade calculation helpers.
- **`docs/07-technical-architecture-save-and-roadmap.md`**: Updated architecture summary.
