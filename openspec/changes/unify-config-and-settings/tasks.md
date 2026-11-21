## 1. Consolidate Shipped Game Config RON Files

- [ ] 1.1 Create `SimConfig` (from current `GameConfig` fields) and composite `GameConfigDef { sim, ai, minimap, skirmish, quest_gen }` with `Validate` delegating to each section's `Validate` in `core/src/config/`.
- [ ] 1.2 Merge `assets/config/base/game.ron`, `base/ai.ron`, `base/minimap.ron`, `base/skirmish.ron`, and `quest/generation.ron` into `core/assets/config/game.ron`; move/keep `core/assets/config/block.ron` and `core/assets/config/quests.ron`, and delete the old split files and `assets/config/setting.ron`.
- [ ] 1.3 Define the unified `GameConfig` resource (`sim`, `ai`, `minimap`, `skirmish`, `quest`) in `core/src/config/mod.rs` and update `ConfigPlugin` to load, validate, and hot-reload all three assets (`game.ron`, `block.ron`, `quests.ron`) inside `ConfigState::Loading -> Ready`.

## 2. Migrate User Preferences to `<data_dir>/settings.ron`

- [ ] 2.1 Refactor `GameSetting` → `GameSettings` (`version`, `volume`, `bloom`, `ai_level`, `keys`) implementing `RonDocument` (`SETTINGS_SCHEMA_VERSION = 1`) via `crate::ron_document!`.
- [ ] 2.2 Load `GameSettings` from `<data_dir>/settings.ron` at startup using `save::document::read_document` (falling back to defaults with warning on missing/corrupt file, never entering `ConfigState::Error`).
- [ ] 2.3 Update `save_setting_on_change` (`core/src/config/setting.rs`) to write atomically via `save::document::write_document` to `<data_dir>/settings.ron` (removing `asset_root()` and `assets/config/setting.ron` writes, and removing `GameSetting` from `AssetServer`).

## 3. Update Call Sites and Fix Skirmish Repeat Spawning

- [ ] 3.1 Replace `GameConfigValue`, `AiConfigValue`, `MinimapConfigValue`, and `GameSetting` usages across `core/src/` with `Res<GameConfig>` (`.sim`, `.ai`, `.minimap`, `.skirmish`, `.quest`) and `Res<GameSettings>`.
- [ ] 3.2 Remove `SkirmishAsset` loading/removal from `core/src/scene/game/skirmish/mod.rs` and read `GameConfig.skirmish` directly so repeated Skirmish sessions spawn enemies cleanly.

## 4. Verification and Docs

- [ ] 4.1 Add unit tests verifying: consolidated `game.ron` parses & validates all sections, invalid sections fail loudly, `GameSettings` round-trips atomically in a temp `data_dir` without touching `assets/`, missing `settings.ron` falls back to defaults, and repeat Skirmish spawns succeed.
- [ ] 4.2 Update `docs/config.md`, `docs/architecture.md`, `docs/save.md`, and `docs/scenes.md` (spec only, no notes) documenting `GameConfig` (`game.ron`, `block.ron`, `quests.ron`) vs. `GameSettings` (`<data_dir>/settings.ron`).
