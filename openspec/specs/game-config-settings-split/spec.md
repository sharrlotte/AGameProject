# game-config-settings-split Specification

## Purpose
TBD - created by archiving change unify-config-and-settings. Update Purpose after archive.
## Requirements
### Requirement: Read-only GameConfig is consolidated and strictly separated from user GameSettings
The engine SHALL separate immutable shipped game configuration (`GameConfig` loaded from `core/assets/config/`) from mutable user preferences (`GameSettings` stored in `<data_dir>/settings.ron`). Small shipped tuning sections (`sim`, `ai`, `minimap`, `skirmish`, `quest_gen`) SHALL live in `core/assets/config/game.ron`, while large content catalogs SHALL remain in `core/assets/config/block.ron` and `core/assets/config/quests.ron`. The game SHALL NEVER write user preferences into the `assets/` directory.

#### Scenario: Boot gate loads and validates all three shipped config files into GameConfig and BlockRegistry
- **WHEN** the application starts with valid `game.ron`, `block.ron`, and `quests.ron`
- **THEN** `ConfigPlugin` validates all three assets before entering `ConfigState::Ready`, populates the single `GameConfig` resource and `BlockRegistry`, and requires no separate asset-loading gates in `QuestPlugin` or `SkirmishPlugin`

#### Scenario: Editing user settings persists atomically to data_dir without touching assets
- **WHEN** the player changes volume, bloom, AI difficulty, or keybindings in Settings
- **THEN** `GameSettings` is written atomically via `save/document.rs` (`RonDocument` + `atomic_write`) to `<data_dir>/settings.ron` and no file inside `assets/` is created or modified

#### Scenario: Missing or corrupt settings.ron falls back to defaults without blocking boot
- **WHEN** `<data_dir>/settings.ron` is missing or malformed at startup
- **THEN** the engine logs a warning, uses `GameSettings::default()`, and still reaches `ConfigState::Ready` and `SceneState::Home`

#### Scenario: Repeat Skirmish sessions spawn enemies from GameConfig.skirmish
- **WHEN** a player runs Quick Battle / Skirmish, returns to Home, and starts a second Skirmish in the same app run
- **THEN** the second Skirmish reads `GameConfig.skirmish` and spawns the configured enemies identically to the first run

