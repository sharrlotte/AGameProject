## Why

Shipped read-only game balance configs and user-editable preferences are currently fragmented and conflated:
- Small shipped balance configs are scattered across 6 separate RON files (`base/game.ron`, `base/ai.ron`, `base/minimap.ron`, `base/skirmish.ron`, `quest/generation.ron`) loaded across **3 different plugins** (`ConfigPlugin`, `QuestPlugin`, and `SkirmishPlugin` — where `SkirmishPlugin` removes its `SkirmishAsset` handle after the first spawn, breaking repeat Skirmish starts!).
- User-editable `GameSetting` (`volume`, `bloom`, `ai_level`, `keys`) lives inside `core/assets/config/setting.ron` alongside read-only game assets, and `save_setting_on_change` (`core/src/config/setting.rs:143-167`) writes user edits directly back into `assets/config/setting.ron` using raw `fs::write` (bypassing `save/document.rs` atomic writes, failing on read-only installs, and dirtying the shipped asset tree).

## What Changes

- Strictly separate **read-only shipped Game Config** (`assets/config/`, hot-reloaded in dev, never modified by gameplay) from **user-editable Game Settings** (`<data_dir>/settings.ron`, persisted atomically via `save/document.rs` `RonDocument`).
- **BREAKING** Merge the small read-only RON configs (`sim`, `ai`, `minimap`, `skirmish`, and `quest_gen` / `worldgen`) into one structured **`core/assets/config/game.ron`**, keeping the two large content catalogs as dedicated RON files (`core/assets/config/block.ron` and `core/assets/config/quests.ron`). Remove `base/ai.ron`, `base/minimap.ron`, `base/skirmish.ron`, `quest/generation.ron`, and `assets/config/setting.ron`.
- Expose one unified read-only **`GameConfig`** Bevy `Resource` (holding `sim`, `ai`, `minimap`, `skirmish`, `quest`) populated and validated inside `ConfigPlugin`'s single boot gate (`ConfigState::Loading -> Ready`) alongside `BlockRegistry` — eliminating `AiConfigValue`, `MinimapConfigValue`, `GameConfigValue` wrapper sprawl, `SkirmishAsset`'s single-use removal bug, and duplicate asset loaders.
- Move **`GameSettings`** (`version`, `volume`, `bloom`, `ai_level`, `keys`) into `<data_dir>/settings.ron` managed by `save/document.rs` (`RonDocument` + `atomic_write`), loading cleanly with defaults if missing or invalid without blocking boot on `ConfigState::Error`.

## Capabilities

### New Capabilities
- `game-config-settings-split`: Strict separation between read-only shipped `GameConfig` (consolidated `assets/config/game.ron` + `block.ron` + `quests.ron`, unified `ConfigPlugin` boot gate) and user-editable `GameSettings` stored atomically in `<data_dir>/settings.ron`.

### Modified Capabilities
- `config-validation`: Validate all consolidated `game.ron` sections (`sim`, `ai`, `minimap`, `skirmish`, `quest_gen`) plus `block.ron` and `quests.ron` under `ConfigPlugin`'s single boot gate, removing legacy mission references and fixing `SkirmishDef` reload/repeat-spawn handling.
- `modular-data-definitions`: Clarify that Rust schema modules remain single-responsibility per domain (`config/sim.rs`, `config/ai/`, `config/minimap.rs`, `config/skirmish.rs`, `config/setting.rs`) while small shipped RON tuning sections consolidate into `assets/config/game.ron` and large catalogs (`block.ron`, `quests.ron`) stay in separate RON files.

## Non-Goals

- Merging `block.ron` (block catalog) or `quests.ron` (authored quest catalog) into `game.ron` — those remain standalone RON catalogs for maintainability.
- Changing individual gameplay physics formulas, AI tuning values, or keybinding UI layout.

## Impact

- `core/assets/config/` (merge small RONs into `game.ron`; relocate `block.ron` / `quests.ron` cleanly; remove `setting.ron` from `assets/`).
- `core/src/config/` (`mod.rs`, `game/`, `ai/`, `minimap/`, `setting.rs`, new `skirmish.rs`).
- `core/src/scene/game/skirmish/mod.rs` (read `GameConfig.skirmish` instead of removing `SkirmishAsset`).
- `core/src/save/document.rs` (`GameSettings` `RonDocument` macro + `settings_path()`).
- Call sites reading `GameConfigValue`, `AiConfigValue`, `MinimapConfigValue`, `GameSetting`.
- `docs/config.md`, `docs/architecture.md`, `docs/save.md`, `docs/scenes.md` (spec-only updates).
