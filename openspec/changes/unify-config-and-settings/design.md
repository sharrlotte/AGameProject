## Context

Inspecting `core/src/config/mod.rs:78-257`, `core/src/config/setting.rs:19-176`, and `core/assets/config/` revealed:
1. Five small tuning RON files (`base/game.ron`, `base/ai.ron`, `base/minimap.ron`, `base/skirmish.ron`, `quest/generation.ron`) are loaded separately across `ConfigPlugin`, `QuestPlugin`, and `SkirmishPlugin`, each with its own handle struct, loader registration, and hot-reload bridge.
2. `SkirmishPlugin` (`scene/game/skirmish/mod.rs:127`) deletes `SkirmishAsset` after spawning once, so launching a second Quick Battle or New World in the same session spawns zero enemies!
3. `GameSetting` (`config/setting.rs`) is loaded via `AssetServer` from `assets/config/setting.ron` and saved back to `assets/config/setting.ron` via non-atomic `fs::write` — mixing mutable user preferences into the read-only shipped asset folder and contradicting `docs/save.md:43` / `world-save/spec.md:20` (which specified `<data_dir>/settings.ron`).

## Goals / Non-Goals

**Goals:**
- Separate **read-only shipped Game Config** (authored by developers, never modified by the game at runtime) from **mutable User Settings** (authored by the player, stored atomically in `<data_dir>/settings.ron`).
- Consolidate small read-only RON tuning files into `assets/config/game.ron` (`sim`, `ai`, `minimap`, `skirmish`, `quest_gen`) while keeping large catalogs (`block.ron`, `quests.ron`) as dedicated RON files.
- Load, validate, and hot-reload all 3 game config RON files inside one `ConfigPlugin` boot gate exposing one unified `GameConfig` resource (plus `BlockRegistry`).
- Fix `Skirmish` repeat-spawning cleanly by reading `GameConfig.skirmish` directly.
- Persist `GameSettings` atomically via `save/document.rs` (`RonDocument` + `atomic_write`, debounced off-thread or non-corrupting) at `<data_dir>/settings.ron`.

**Non-Goals:**
- Merging `block.ron` or `quests.ron` into `game.ron`.
- Adding new settings UI sliders beyond existing `volume`, `bloom`, `ai_level`, and `keys`.

## Decisions

### 1. Disk Layout: 3 Read-Only Game Config RONs vs. 1 User Settings RON
- **Choice**:
  - **Shipped read-only assets (`core/assets/config/`)**:
    - `config/game.ron` → `GameConfigDef { sim: SimConfig, ai: AiConfig, minimap: MinimapConfig, skirmish: SkirmishConfig, quest_gen: QuestGenConfig }` (with `#[serde(deny_unknown_fields)]` and composite `Validate` delegating to each section's `Validate`).
    - `config/block.ron` → `BlocksConfig` (builds `BlockRegistry`).
    - `config/quests.ron` → `QuestCatalogDef` (populates `GameConfig.quest.authored`).
    - `config/starter/*.ron` → embedded starter `ShipDesign`s.
  - **User preferences (`<data_dir>/settings.ron`)**:
    - `GameSettings { version: u32, volume: f32, bloom: bool, ai_level: u8, keys: KeySettings }` implementing `RonDocument` (`SETTINGS_SCHEMA_VERSION = 1`) via `crate::ron_document!`, read at startup with `read_document` (falling back to `GameSettings::default()` with a warning if missing or corrupt) and written atomically via `write_document` (`atomic_write`) on debounced UI change. Never touches `assets/`.
- **Rationale**: Players can safely reset or back up `<data_dir>/settings.ron` alongside saves without corrupting read-only install files; developers edit one concise `game.ron` for gameplay tuning while large tables (`block.ron`, `quests.ron`) remain isolated and modular.
- **Alternative Considered**: *Keep all 6 small RON files separate* — user explicitly preferred merging the small RON files into `game.ron` while keeping `block.ron` and `quests.ron` separate.

### 2. Unified `GameConfig` Resource + Modular Rust Source Files
- **Choice**:
  - Keep each Rust schema/validation module in its own focused file under `core/src/config/` (`sim.rs`, `ai/`, `minimap.rs`, `skirmish.rs`, `block/`, `setting.rs`), respecting `modular-data-definitions` and `AGENTS.md` file size limits.
  - Expose one read-only `GameConfig` resource in `core/src/config/mod.rs`:
    ```rust
    #[derive(Resource, Clone, Debug, Default)]
    pub struct GameConfig {
        pub sim: SimConfig,
        pub ai: AiConfig,
        pub minimap: MinimapConfig,
        pub skirmish: SkirmishConfig,
        pub quest: QuestConfig, // authored (from quests.ron) + generation (from game.ron)
    }
    ```
  - `ConfigPlugin` loads all 3 asset handles (`game.ron`, `block.ron`, `quests.ron`), validates all 3 before `ConfigState::Ready`, and runs 3 hot-reload bridges (`game_bridge`, `blocks_bridge`, `quests_bridge`).
  - `GameSettings` is loaded synchronously once in `PreStartup`/`Startup` (tiny <200B file, or via `IoWorker` on save) and managed by a dedicated `SettingsPlugin` (or `config/setting.rs`) cleanly decoupled from `ConfigState::Error`.

### 3. Fixing `Skirmish` Repeat-Spawn Bug
- **Choice**: Delete `SkirmishAsset` loading from `core/src/scene/game/skirmish/mod.rs` and read `&config.skirmish` directly from `Res<GameConfig>`, guaranteeing every Quick Battle start spawns enemies reliably across unlimited sessions.

## Risks / Trade-offs

- **Call-site churn (`GameConfigValue` / `AiConfigValue` / `MinimapConfigValue` → `GameConfig`)** → Pure mechanical path updates (`config.0.thrust_power_factor` → `config.sim.thrust_power_factor`, `ai_cfg.0` → `config.ai`, `minimap.0` → `config.minimap`), making system signatures significantly shorter (`Res<GameConfig>` instead of 3–4 separate config resources!).
- **Hot-reload `resource_changed::<GameConfig>` granularity** → Only HUD/config consumers check `resource_changed` on config; hot reload is a rare dev-only event so updating `GameConfig` on `game.ron` edit has zero production cost.
- **Settings file migration from old `assets/config/setting.ron`** → Hard cutover: if `<data_dir>/settings.ron` does not exist on first launch, initialize defaults and write `<data_dir>/settings.ron` when changed (never read/write `assets/config/setting.ron`).
